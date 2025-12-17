# ps3-update-core

A Rust library for fetching and downloading PS3 game updates from Sony's official PlayStation Network servers.

## Features

- ✅ Fetch available updates for any PS3 game by Title ID
- ✅ Download update packages with real-time progress tracking
- ✅ Support for single-threaded and multi-part concurrent downloads
- ✅ Extract game metadata (title, version, size, SHA1 hash)
- ✅ Comprehensive error handling
- ✅ Framework-agnostic (works with any UI framework or as standalone)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
ps3-update-core = { path = "../ps3-update-core" }
tokio = { version = "1", features = ["full"] }
```

## Quick Start

```rust
use ps3_update_core::{UpdateFetcher, DownloadManager, DownloadMode};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create fetcher
    let fetcher = UpdateFetcher::new()?;

    // Check if server is online
    if !fetcher.check_server_status().await {
        eprintln!("PS3 update server is not reachable");
        return Ok(());
    }

    // Fetch updates for God of War III (BLES00799)
    let result = fetcher.fetch_updates("BLES00799").await?;

    println!("Game: {}", result.game_title);
    println!("Found {} updates:", result.results.len());

    for pkg in &result.results {
        println!("  Version {} - {} - SHA1: {}",
            pkg.version, pkg.size_human, pkg.sha1);
    }

    // Download the latest update
    if let Some(latest) = result.results.first() {
        println!("\nDownloading {}...", latest.filename);

        let manager = DownloadManager::new()?;
        let job_id = manager.start_download(
            &latest.url,
            PathBuf::from(format!("/tmp/{}", latest.filename)),
            DownloadMode::MultiPart { num_parts: 4 },
        ).await?;

        // Monitor progress
        loop {
            let progress = manager.get_progress(&job_id)?;

            print!("\rProgress: {:.1}% - {} - {}     ",
                progress.percent,
                progress.speed_human,
                progress.filename.as_deref().unwrap_or(""));

            if progress.done {
                println!();
                if let Some(err) = progress.error {
                    eprintln!("Error: {}", err);
                } else {
                    println!("Download complete!");
                }
                break;
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }

    Ok(())
}
```

## API Documentation

### UpdateFetcher

Fetches PS3 game update information from Sony's servers.

```rust
// Create new fetcher
let fetcher = UpdateFetcher::new()?;

// Check if PS3 server is reachable
let online = fetcher.check_server_status().await;

// Fetch updates for a title (e.g., "BLES00779", "NPUA80662")
let result = fetcher.fetch_updates("BLES00779").await?;
```

### DownloadManager

Manages download jobs with progress tracking.

```rust
// Create new download manager
let manager = DownloadManager::new()?;

// Start a download (returns job ID)
let job_id = manager.start_download(
    "https://...",
    PathBuf::from("/path/to/file.pkg"),
    DownloadMode::Direct,
).await?;

// Get progress
let progress = manager.get_progress(&job_id)?;

// Clean up completed job
manager.remove_job(&job_id);
```

### Download Modes

```rust
// Single-threaded download
DownloadMode::Direct

// Multi-part concurrent download (4 parts)
DownloadMode::MultiPart { num_parts: 4 }
```

Multi-part downloads automatically fall back to single-threaded if the server doesn't support range requests.

### Types

#### PackageInfo
Contains information about a PS3 update package:
- `version` - Update version (e.g., "1.01")
- `system_ver` - Required PS3 system version
- `size_bytes` - Size in bytes
- `size_human` - Human-readable size (e.g., "245.67 MB")
- `url` - Direct download URL
- `sha1` - SHA1 hash for verification
- `filename` - Package filename

#### FetchResult
Result of fetching updates:
- `results` - Vector of `PackageInfo` (sorted by version, newest first)
- `error` - Optional error message
- `game_title` - Game name
- `cleaned_title_id` - Normalized title ID

#### ProgressInfo
Download progress information:
- `filename` - File being downloaded
- `total` - Total size in bytes
- `downloaded` - Bytes downloaded so far
- `percent` - Completion percentage (0-100)
- `speed_bytes_per_sec` - Download speed
- `speed_human` - Human-readable speed (e.g., "2.5 MB/s")
- `done` - Whether download is complete
- `error` - Optional error message

### Utility Functions

```rust
use ps3_update_core::{format_size, clean_title_id, safe_dir_name};

// Format bytes to human-readable
let size = format_size(123456789); // "117.74 MB"

// Clean title ID
let clean = clean_title_id("BLES-00779"); // "BLES00779"

// Safe directory name
let dir = safe_dir_name("Game: Test!"); // "Game Test"
```

## Error Handling

The library uses `PS3UpdateError` for all errors:

```rust
use ps3_update_core::PS3UpdateError;

match fetcher.fetch_updates("INVALID").await {
    Ok(result) => println!("Success!"),
    Err(PS3UpdateError::Network(e)) => eprintln!("Network error: {}", e),
    Err(PS3UpdateError::XmlParse(e)) => eprintln!("Parse error: {}", e),
    Err(PS3UpdateError::NoUpdatesFound(id)) => eprintln!("No updates for {}", id),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Finding PS3 Title IDs

PS3 Title IDs are 9-character codes like:
- `BLES00779` - Uncharted: Drake's Fortune (EU)
- `BLUS30060` - Uncharted: Drake's Fortune (US)
- `NPUA80662` - PlayStation Home (US)

Search for game IDs at [SerialStation.com](https://serialstation.com/)

## License

MIT
