//! Example of fetching updates for multiple games in batch
//!
//! Run with: cargo run --example batch_fetch

use ps3_update_core::UpdateFetcher;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== PS3 Update Core - Batch Fetch Example ===\n");

    let fetcher = UpdateFetcher::new()?;

    // List of popular PS3 games to check
    let games = vec![
        "BLES00779", // Uncharted: Drake's Fortune
        "BLES00806", // Metal Gear Solid 4
        "BLES00932", // LittleBigPlanet
        "BLES00510", // Resistance 2
        "BCES00019", // MotorStorm
    ];

    println!("Fetching updates for {} games...\n", games.len());

    for title_id in games {
        print!("Checking {}... ", title_id);

        match fetcher.fetch_updates(title_id).await {
            Ok(result) => {
                if result.results.is_empty() {
                    println!("No updates found");
                } else {
                    let latest = &result.results[0];
                    println!(
                        "✓ {} - Latest: v{} ({})",
                        result.game_title, latest.version, latest.size_human
                    );
                }
            }
            Err(e) => {
                println!("✗ Error: {}", e);
            }
        }
    }

    println!("\nDone!");
    Ok(())
}
