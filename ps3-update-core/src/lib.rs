//! # ps3-update-core
//!
//! A Rust library for fetching and downloading PS3 game updates from Sony's official servers.
//!
//! ## Features
//!
//! - Fetch available updates for any PS3 game by Title ID
//! - Download update packages with progress tracking
//! - Support for both single-threaded and multi-part downloads
//! - Extract game metadata (title, version, size, SHA1 hash)
//!
//! ## Example
//!
//! ```no_run
//! use ps3_update_core::{UpdateFetcher, DownloadManager, DownloadMode};
//! use std::path::PathBuf;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Fetch updates for a game
//!     let fetcher = UpdateFetcher::new()?;
//!     let result = fetcher.fetch_updates("BLES00779").await?;
//!
//!     println!("Found {} updates for {}", result.results.len(), result.game_title);
//!
//!     // Download the first update
//!     if let Some(pkg) = result.results.first() {
//!         let manager = DownloadManager::new()?;
//!         let job_id = manager.start_download(
//!             &pkg.url,
//!             PathBuf::from("/tmp/update.pkg"),
//!             DownloadMode::Direct,
//!         ).await?;
//!
//!         // Poll for progress
//!         loop {
//!             let progress = manager.get_progress(&job_id)?;
//!             println!("Progress: {:.1}%", progress.percent);
//!             if progress.done {
//!                 break;
//!             }
//!             tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```

pub mod downloader;
pub mod fetcher;
pub mod types;
pub mod utils;

// Re-export main types for convenience
pub use downloader::DownloadManager;
pub use fetcher::UpdateFetcher;
pub use types::{
    DownloadMode, FetchResult, PS3UpdateError, PackageInfo, ProgressInfo, Result,
};
pub use utils::{clean_title_id, format_size, safe_dir_name};
