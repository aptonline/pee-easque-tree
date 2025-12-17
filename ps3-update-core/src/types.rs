use serde::{Deserialize, Serialize};

/// Represents a single PS3 update package
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

/// Result of fetching updates for a title
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchResult {
    pub results: Vec<PackageInfo>,
    pub error: Option<String>,
    pub game_title: String,
    pub cleaned_title_id: String,
}

/// Download progress information
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

/// Download mode: single-threaded or multi-part
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadMode {
    Direct,
    MultiPart { num_parts: usize },
}

impl Default for DownloadMode {
    fn default() -> Self {
        DownloadMode::Direct
    }
}

/// Error types for the library
#[derive(Debug, thiserror::Error)]
pub enum PS3UpdateError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("XML parsing error: {0}")]
    XmlParse(String),

    #[error("Invalid title ID: {0}")]
    InvalidTitleId(String),

    #[error("No updates found for title ID: {0}")]
    NoUpdatesFound(String),

    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),

    #[error("Download error: {0}")]
    Download(String),

    #[error("Job not found: {0}")]
    JobNotFound(String),
}

pub type Result<T> = std::result::Result<T, PS3UpdateError>;
