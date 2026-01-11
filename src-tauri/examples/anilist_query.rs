// src-tauri/examples/anilist_query.rs
use animehub::integrations::anilist::client::AniListClient;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = AniListClient::new();
    let results = client.search_anime("Naruto").await?;
    println!("Found {} results", results.len());
    for a in results {
        println!(
            "{} - episodes: {:?}",
            a.title.romaji.unwrap_or_default(),
            a.episodes
        );
    }
    Ok(())
}
