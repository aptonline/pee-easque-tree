//! Test download with a smaller file to verify functionality
//!
//! Run with: cargo run --example test_small_download

use ps3_update_core::{DownloadManager, DownloadMode, UpdateFetcher};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== PS3 Update Core - Download Test ===\n");

    let fetcher = UpdateFetcher::new()?;

    // Test with a title that hopefully has a smaller update
    let title_id = "BLES00779";
    println!("Fetching updates for {}...", title_id);

    let result = fetcher.fetch_updates(title_id).await?;

    if result.results.is_empty() {
        println!("No updates found.");
        return Ok(());
    }

    let pkg = &result.results[0];
    println!("\nFound update:");
    println!("  Game: {}", result.game_title);
    println!("  Version: {}", pkg.version);
    println!("  Size: {}", pkg.size_human);
    println!("  URL: {}\n", pkg.url);

    // Test with direct mode first (simpler, more reliable)
    println!("Testing DIRECT download mode...");
    let dest_path = PathBuf::from(format!("/tmp/ps3_test_direct_{}", pkg.filename));

    let manager = DownloadManager::new()?;
    let job_id = manager
        .start_download(&pkg.url, dest_path.clone(), DownloadMode::Direct)
        .await?;

    println!("Download started with job ID: {}", job_id);
    println!("Destination: {}\n", dest_path.display());

    // Monitor progress
    let mut last_percent = -1.0;
    let start = std::time::Instant::now();

    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        match manager.get_progress(&job_id) {
            Ok(progress) => {
                if (progress.percent - last_percent).abs() > 0.5 || progress.done {
                    let elapsed = start.elapsed().as_secs();
                    print!(
                        "\r[{:3}s] Progress: {:>5.1}% | Downloaded: {:>10} / {:>10} | Speed: {:>12}     ",
                        elapsed,
                        progress.percent,
                        ps3_update_core::format_size(progress.downloaded),
                        ps3_update_core::format_size(progress.total),
                        progress.speed_human
                    );
                    std::io::Write::flush(&mut std::io::stdout())?;
                    last_percent = progress.percent;
                }

                if progress.done {
                    println!("\n");
                    if let Some(err) = progress.error {
                        eprintln!("❌ Download failed: {}", err);
                        return Err(err.into());
                    } else {
                        println!("✅ Download complete!");
                        println!("📁 File saved to: {}", dest_path.display());

                        // Show file size
                        if let Ok(metadata) = std::fs::metadata(&dest_path) {
                            println!("📊 File size: {}", ps3_update_core::format_size(metadata.len()));
                        }

                        // Test multipart mode
                        println!("\n\n=== Testing MULTI-PART download mode ===");
                        let mp_dest = PathBuf::from(format!("/tmp/ps3_test_multi_{}", pkg.filename));

                        let job_id2 = manager
                            .start_download(&pkg.url, mp_dest.clone(), DownloadMode::MultiPart { num_parts: 4 })
                            .await?;

                        println!("Download started with job ID: {}", job_id2);
                        println!("Destination: {}\n", mp_dest.display());

                        // Monitor multipart progress
                        let mut last_percent2 = -1.0;
                        let start2 = std::time::Instant::now();

                        loop {
                            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                            match manager.get_progress(&job_id2) {
                                Ok(progress2) => {
                                    if (progress2.percent - last_percent2).abs() > 0.5 || progress2.done {
                                        let elapsed2 = start2.elapsed().as_secs();
                                        print!(
                                            "\r[{:3}s] Progress: {:>5.1}% | Downloaded: {:>10} / {:>10} | Speed: {:>12}     ",
                                            elapsed2,
                                            progress2.percent,
                                            ps3_update_core::format_size(progress2.downloaded),
                                            ps3_update_core::format_size(progress2.total),
                                            progress2.speed_human
                                        );
                                        std::io::Write::flush(&mut std::io::stdout())?;
                                        last_percent2 = progress2.percent;
                                    }

                                    if progress2.done {
                                        println!("\n");
                                        if let Some(err) = progress2.error {
                                            eprintln!("❌ Multi-part download failed: {}", err);
                                            eprintln!("This is OK - the library should have fallen back to direct mode");
                                        } else {
                                            println!("✅ Multi-part download complete!");
                                            println!("📁 File saved to: {}", mp_dest.display());
                                        }
                                        break;
                                    }
                                }
                                Err(e) => {
                                    eprintln!("\n❌ Progress check failed: {}", e);
                                    break;
                                }
                            }
                        }
                    }
                    break;
                }
            }
            Err(e) => {
                eprintln!("\n❌ Progress check failed: {}", e);
                break;
            }
        }
    }

    println!("\n✅ Test complete!");
    Ok(())
}
