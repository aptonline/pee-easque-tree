//! Standalone example of using ps3-update-core without Tauri
//!
//! Run with: cargo run --example standalone

use ps3_update_core::{DownloadManager, DownloadMode, UpdateFetcher};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== PS3 Update Core - Standalone Example ===\n");

    // Create fetcher
    let fetcher = UpdateFetcher::new()?;

    // Check server status
    println!("Checking PS3 server status...");
    let online = fetcher.check_server_status().await;
    println!("Server status: {}\n", if online { "✓ Online" } else { "✗ Offline" });

    if !online {
        println!("Cannot proceed: PS3 update server is not reachable");
        return Ok(());
    }

    // Example: Fetch updates for Uncharted: Drake's Fortune (EU)
    let title_id = "BLES00779";
    println!("Fetching updates for {}...", title_id);

    let result = fetcher.fetch_updates(title_id).await?;

    println!("\n📀 Game: {}", result.game_title);
    println!("📝 Title ID: {}", result.cleaned_title_id);
    println!("📦 Found {} update(s):\n", result.results.len());

    if result.results.is_empty() {
        println!("No updates available for this title.");
        return Ok(());
    }

    // Display all available updates
    for (i, pkg) in result.results.iter().enumerate() {
        println!("  {}. Version: {}", i + 1, pkg.version);
        println!("     Size: {}", pkg.size_human);
        println!("     System Ver: {}", pkg.system_ver);
        println!("     SHA1: {}", pkg.sha1);
        println!("     Filename: {}", pkg.filename);
        println!();
    }

    // Ask user which update to download
    println!("Would you like to download the latest update? (y/n)");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if !input.trim().eq_ignore_ascii_case("y") {
        println!("Skipping download.");
        return Ok(());
    }

    // Download the first (latest) update
    let latest = &result.results[0];
    let dest_path = PathBuf::from(format!("/tmp/{}", latest.filename));

    println!("\n⬇️  Downloading to: {}", dest_path.display());
    println!("Using multi-part download (4 parts)...\n");

    let manager = DownloadManager::new()?;
    let job_id = manager
        .start_download(
            &latest.url,
            dest_path.clone(),
            DownloadMode::MultiPart { num_parts: 4 },
        )
        .await?;

    // Monitor progress
    let mut last_percent = -1.0;
    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        let progress = manager.get_progress(&job_id)?;

        // Only update display if percentage changed significantly
        if (progress.percent - last_percent).abs() > 0.1 || progress.done {
            print!(
                "\r📊 Progress: {:>5.1}% | Downloaded: {:>10} / {:>10} | Speed: {:>12}     ",
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
                println!("\n🔐 Verify SHA1: {}", latest.sha1);
            }
            break;
        }
    }

    Ok(())
}
