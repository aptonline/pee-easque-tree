use once_cell::sync::Lazy;
use ps3_update_core::{DownloadManager, DownloadMode, UpdateFetcher};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

// Global state for download manager
static DOWNLOAD_MANAGER: Lazy<Mutex<Option<Arc<DownloadManager>>>> = Lazy::new(|| Mutex::new(None));

// Track file paths for cleanup on cancel
static DOWNLOAD_PATHS: Lazy<Mutex<HashMap<String, PathBuf>>> = Lazy::new(|| Mutex::new(HashMap::new()));

// Types for frontend communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub version: String,
    pub system_ver: String,
    pub size_bytes: u64,
    pub size_human: String,
    pub url: String,
    pub sha1: String,
    pub filename: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchResult {
    pub results: Vec<PackageInfo>,
    pub error: Option<String>,
    pub game_title: String,
    pub cleaned_title_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressInfo {
    pub filename: Option<String>,
    pub total: u64,
    pub downloaded: u64,
    pub percent: f64,
    pub speed_bytes_per_sec: f64,
    pub speed_human: String,
    pub done: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub download_path: String,
}

// Convert ps3_update_core types to our types
impl From<ps3_update_core::PackageInfo> for PackageInfo {
    fn from(pkg: ps3_update_core::PackageInfo) -> Self {
        PackageInfo {
            version: pkg.version,
            system_ver: pkg.system_ver,
            size_bytes: pkg.size_bytes,
            size_human: pkg.size_human,
            url: pkg.url,
            sha1: pkg.sha1,
            filename: pkg.filename,
        }
    }
}

impl From<ps3_update_core::FetchResult> for FetchResult {
    fn from(result: ps3_update_core::FetchResult) -> Self {
        FetchResult {
            results: result.results.into_iter().map(|p| p.into()).collect(),
            error: result.error,
            game_title: result.game_title,
            cleaned_title_id: result.cleaned_title_id,
        }
    }
}

impl From<ps3_update_core::ProgressInfo> for ProgressInfo {
    fn from(progress: ps3_update_core::ProgressInfo) -> Self {
        ProgressInfo {
            filename: progress.filename,
            total: progress.total,
            downloaded: progress.downloaded,
            percent: progress.percent,
            speed_bytes_per_sec: progress.speed_bytes_per_sec,
            speed_human: progress.speed_human,
            done: progress.done,
            error: progress.error,
        }
    }
}

#[tauri::command]
async fn check_server_status() -> Result<bool, String> {
    let fetcher = UpdateFetcher::new().map_err(|e| e.to_string())?;
    Ok(fetcher.check_server_status().await)
}

#[tauri::command]
async fn fetch_updates(title_id: String) -> Result<FetchResult, String> {
    let fetcher = UpdateFetcher::new().map_err(|e| e.to_string())?;
    let result = fetcher.fetch_updates(&title_id).await.map_err(|e| e.to_string())?;
    Ok(result.into())
}

#[tauri::command]
async fn start_download(
    url: String,
    filename: String,
    download_path: String,
    game_title: String,
    title_id: String,
    multi_part: bool,
) -> Result<String, String> {
    // Initialize download manager if needed and get an Arc clone
    let manager = {
        let mut manager_lock = DOWNLOAD_MANAGER.lock().unwrap();
        if manager_lock.is_none() {
            *manager_lock = Some(Arc::new(DownloadManager::new().map_err(|e| e.to_string())?));
        }
        manager_lock.as_ref().unwrap().clone()
    };

    // Create subfolder: "GameTitle (TITLEID)"
    let folder_name = format!("{} ({})", game_title, title_id);
    let safe_folder_name = folder_name
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect::<String>();

    let subfolder = PathBuf::from(download_path).join(safe_folder_name);
    let path = subfolder.join(&filename);

    let mode = if multi_part {
        DownloadMode::MultiPart { num_parts: 4 }
    } else {
        DownloadMode::Direct
    };

    let job_id = manager
        .start_download(&url, path.clone(), mode)
        .await
        .map_err(|e| e.to_string())?;

    // Track the file path for cleanup
    {
        let mut paths = DOWNLOAD_PATHS.lock().unwrap();
        paths.insert(job_id.clone(), path);
    }

    Ok(job_id)
}

#[tauri::command]
async fn cancel_download(job_id: String) -> Result<(), String> {
    // Remove the job from the manager
    {
        let manager_lock = DOWNLOAD_MANAGER.lock().unwrap();
        if let Some(manager) = manager_lock.as_ref() {
            manager.remove_job(&job_id);
        }
    }

    // Delete the partial file
    let path = {
        let mut paths = DOWNLOAD_PATHS.lock().unwrap();
        paths.remove(&job_id)
    };

    if let Some(file_path) = path {
        if file_path.exists() {
            tokio::fs::remove_file(&file_path)
                .await
                .map_err(|e| format!("Failed to delete partial file: {}", e))?;
        }
    }

    Ok(())
}

#[tauri::command]
fn get_download_progress(job_id: String) -> Result<ProgressInfo, String> {
    let manager_lock = DOWNLOAD_MANAGER.lock().unwrap();
    if let Some(manager) = manager_lock.as_ref() {
        let progress = manager.get_progress(&job_id).map_err(|e| e.to_string())?;
        Ok(progress.into())
    } else {
        Err("Download manager not initialized".to_string())
    }
}

#[tauri::command]
fn remove_download_job(job_id: String) -> Result<(), String> {
    let manager_lock = DOWNLOAD_MANAGER.lock().unwrap();
    if let Some(manager) = manager_lock.as_ref() {
        manager.remove_job(&job_id);
        Ok(())
    } else {
        Err("Download manager not initialized".to_string())
    }
}

#[tauri::command]
fn get_default_download_path() -> String {
    dirs_next::download_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .to_string_lossy()
        .to_string()
}

#[tauri::command]
async fn pick_download_directory(app: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;

    let path = app.dialog()
        .file()
        .blocking_pick_folder();

    Ok(path.map(|p| p.to_string()))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            check_server_status,
            fetch_updates,
            start_download,
            cancel_download,
            get_download_progress,
            remove_download_job,
            get_default_download_path,
            pick_download_directory,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
