use crate::types::{DownloadMode, PS3UpdateError, ProgressInfo, Result};
use crate::utils::format_size;
use futures_util::StreamExt;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};

/// Internal state for a download job
#[derive(Debug, Clone)]
struct JobState {
    filename: String,
    total: u64,
    downloaded: u64,
    start: Instant,
    done: bool,
    error: Option<String>,
}

/// Download manager for PS3 update packages
pub struct DownloadManager {
    client: reqwest::Client,
    jobs: Arc<Mutex<HashMap<String, JobState>>>,
}

impl DownloadManager {
    /// Create a new DownloadManager
    pub fn new() -> Result<Self> {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()?;

        Ok(Self {
            client,
            jobs: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Start a download job and return a job ID for tracking
    pub async fn start_download(
        &self,
        url: &str,
        dest_path: PathBuf,
        mode: DownloadMode,
    ) -> Result<String> {
        let filename = dest_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("update.pkg")
            .to_string();

        // Ensure parent directory exists
        if let Some(parent) = dest_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let job_id = format!("{:x}", rand::random::<u64>());

        {
            let mut jobs = self.jobs.lock().unwrap();
            jobs.insert(
                job_id.clone(),
                JobState {
                    filename: filename.clone(),
                    total: 0,
                    downloaded: 0,
                    start: Instant::now(),
                    done: false,
                    error: None,
                },
            );
        }

        let url = url.to_string();
        let client = self.client.clone();
        let jobs = self.jobs.clone();
        let job_id_clone = job_id.clone();

        tokio::spawn(async move {
            let result = match mode {
                DownloadMode::Direct => {
                    Self::download_direct(&client, &url, &dest_path, &jobs, &job_id_clone).await
                }
                DownloadMode::MultiPart { num_parts } => {
                    // Try multipart, fallback to direct on any error
                    let mp_result = Self::download_multipart(
                        &client,
                        &url,
                        &dest_path,
                        num_parts,
                        &jobs,
                        &job_id_clone,
                    )
                    .await;

                    // If multipart fails, try direct download
                    if mp_result.is_err() {
                        Self::download_direct(&client, &url, &dest_path, &jobs, &job_id_clone).await
                    } else {
                        mp_result
                    }
                }
            };

            if let Err(e) = result {
                let mut jobs = jobs.lock().unwrap();
                if let Some(job) = jobs.get_mut(&job_id_clone) {
                    job.done = true;
                    job.error = Some(e.to_string());
                }
            }
        });

        Ok(job_id)
    }

    /// Get progress information for a job
    pub fn get_progress(&self, job_id: &str) -> Result<ProgressInfo> {
        let jobs = self.jobs.lock().unwrap();

        if let Some(job) = jobs.get(job_id) {
            let total = job.total;
            let downloaded = job.downloaded;
            let percent = if total > 0 {
                (downloaded as f64 / total as f64) * 100.0
            } else {
                0.0
            };

            let elapsed = job.start.elapsed().as_secs_f64().max(0.001);
            let speed = downloaded as f64 / elapsed;
            let speed_human = if speed > 0.0 {
                format!("{}/s", format_size(speed as u64))
            } else {
                "0 B/s".to_string()
            };

            Ok(ProgressInfo {
                filename: Some(job.filename.clone()),
                total,
                downloaded,
                percent,
                speed_bytes_per_sec: speed,
                speed_human,
                done: job.done,
                error: job.error.clone(),
            })
        } else {
            Err(PS3UpdateError::JobNotFound(job_id.to_string()))
        }
    }

    /// Remove a completed job from tracking
    pub fn remove_job(&self, job_id: &str) {
        let mut jobs = self.jobs.lock().unwrap();
        jobs.remove(job_id);
    }

    async fn download_direct(
        client: &reqwest::Client,
        url: &str,
        dest_path: &Path,
        jobs: &Arc<Mutex<HashMap<String, JobState>>>,
        job_id: &str,
    ) -> Result<()> {
        let resp = client.get(url).send().await?;

        if !resp.status().is_success() {
            return Err(PS3UpdateError::Download(format!(
                "HTTP error: {}",
                resp.status()
            )));
        }

        let total_size = resp.content_length().unwrap_or(0);

        {
            let mut jobs = jobs.lock().unwrap();
            if let Some(job) = jobs.get_mut(job_id) {
                job.total = total_size;
            }
        }

        let mut file = tokio::fs::File::create(dest_path).await?;
        let mut stream = resp.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;

            let mut jobs = jobs.lock().unwrap();
            if let Some(job) = jobs.get_mut(job_id) {
                job.downloaded = job.downloaded.saturating_add(chunk.len() as u64);
            }
        }

        let mut jobs = jobs.lock().unwrap();
        if let Some(job) = jobs.get_mut(job_id) {
            job.done = true;
        }

        Ok(())
    }

    async fn download_multipart(
        client: &reqwest::Client,
        url: &str,
        dest_path: &Path,
        num_parts: usize,
        jobs: &Arc<Mutex<HashMap<String, JobState>>>,
        job_id: &str,
    ) -> Result<()> {
        // First, check if server supports range requests
        let head_resp = client.head(url).send().await?;
        let total_size = head_resp
            .content_length()
            .ok_or_else(|| PS3UpdateError::Download("Cannot determine file size".into()))?;

        // Ensure total_size is valid
        if total_size == 0 {
            return Err(PS3UpdateError::Download("File size is zero".into()));
        }

        let accept_ranges = head_resp
            .headers()
            .get("accept-ranges")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_lowercase().contains("bytes"))
            .unwrap_or(false);

        if !accept_ranges {
            return Self::download_direct(client, url, dest_path, jobs, job_id).await;
        }

        {
            let mut jobs = jobs.lock().unwrap();
            if let Some(job) = jobs.get_mut(job_id) {
                job.total = total_size;
            }
        }

        // Calculate ranges
        let part_size = std::cmp::max(total_size / num_parts as u64, 1);
        let mut ranges = Vec::new();
        let mut start = 0;

        for i in 0..num_parts {
            let mut end = start + part_size.saturating_sub(1);
            if i == num_parts - 1 || end >= total_size.saturating_sub(1) {
                end = total_size.saturating_sub(1);
            }
            ranges.push((start, end));
            start = end + 1;
            if start >= total_size {
                break;
            }
        }

        // Pre-create file
        tokio::fs::File::create(dest_path).await?;

        // Download parts concurrently
        let futures = ranges.into_iter().map(|(start, end)| {
            let client = client.clone();
            let url = url.to_string();
            let dest_path = dest_path.to_path_buf();
            let jobs = jobs.clone();
            let job_id = job_id.to_string();

            async move {
                let resp = client
                    .get(&url)
                    .header("Range", format!("bytes={}-{}", start, end))
                    .send()
                    .await?;

                if !resp.status().is_success() && resp.status().as_u16() != 206 {
                    return Err(PS3UpdateError::Download(format!(
                        "Range request failed: {}",
                        resp.status()
                    )));
                }

                let mut stream = resp.bytes_stream();
                let mut file = tokio::fs::OpenOptions::new()
                    .write(true)
                    .open(&dest_path)
                    .await?;

                file.seek(std::io::SeekFrom::Start(start)).await?;

                while let Some(chunk) = stream.next().await {
                    let chunk = chunk?;
                    file.write_all(&chunk).await?;

                    let mut jobs = jobs.lock().unwrap();
                    if let Some(job) = jobs.get_mut(&job_id) {
                        job.downloaded = job.downloaded.saturating_add(chunk.len() as u64);
                    }
                }

                Ok::<(), PS3UpdateError>(())
            }
        });

        let results: Vec<Result<()>> = futures_util::future::join_all(futures).await;

        let mut jobs = jobs.lock().unwrap();
        if let Some(job) = jobs.get_mut(job_id) {
            job.done = true;
            if results.iter().any(|r| r.is_err()) {
                job.error = Some("One or more parts failed".into());
            }
        }

        Ok(())
    }
}

impl Default for DownloadManager {
    fn default() -> Self {
        Self::new().expect("Failed to create DownloadManager")
    }
}
