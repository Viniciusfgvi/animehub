// src-tauri/examples/playback_e2e_test.rs
//
// PHASE 3 VALIDATION TEST: Post-Resolution Phase (Playback)
//
// PURPOSE:
// - Validate that playback works after explicit domain resolution
// - Flow: Scan → Create Anime (via service) → Create Episode (via service) →
//         Link File (via service) → Start Playback → Stop Playback
// - Validate MPV opens and closes cleanly
//
// CONTRACT REFERENCES:
// - SERVICE_CONTRACTS.md Section 1: Anime Service
// - SERVICE_CONTRACTS.md Section 2: Episode Service
// - SERVICE_CONTRACTS.md Section 3: File Service
// - SERVICE_CONTRACTS.md Section 5: Playback Service
//
// CRITICAL:
// - Domain entities are created EXPLICITLY via services
// - No implicit inference from file names
// - This is the canonical flow for playback

use std::path::PathBuf;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

use animehub::db::{create_connection_pool, initialize_database};
use animehub::domain::anime::{AnimeStatus, AnimeType};
use animehub::domain::episode::EpisodeNumber;
use animehub::domain::file::FileType;
use animehub::events::EventBus;
use animehub::integrations::MpvClient;
use animehub::repositories::*;
use animehub::services::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== PLAYBACK E2E TEST ===");
    println!("Purpose: Validate post-resolution playback flow");
    println!();

    // =========================================================================
    // 1. INFRASTRUCTURE BOOTSTRAP (same as main.rs)
    // =========================================================================
    println!("[SETUP] Bootstrapping infrastructure...");

    let event_bus = Arc::new(EventBus::new());
    let pool = Arc::new(create_connection_pool()?);
    let mpv_client = Arc::new(MpvClient::new()?);

    // Initialize schema (idempotent)
    {
        let conn = pool.get()?;
        initialize_database(&conn)?;
    }
    println!("[SETUP] Database initialized.");

    // =========================================================================
    // 2. REPOSITORIES
    // =========================================================================
    let anime_repo: Arc<dyn AnimeRepository> = Arc::new(SqliteAnimeRepository::new(pool.clone()));
    let episode_repo: Arc<dyn EpisodeRepository> =
        Arc::new(SqliteEpisodeRepository::new(pool.clone()));
    let file_repo: Arc<dyn FileRepository> = Arc::new(SqliteFileRepository::new(pool.clone()));
    let anime_alias_repo: Arc<dyn AnimeAliasRepository> =
        Arc::new(SqliteAnimeAliasRepository::new(pool.clone()));
    let external_ref_repo: Arc<dyn ExternalReferenceRepository> =
        Arc::new(SqliteExternalReferenceRepository::new(pool.clone()));

    // =========================================================================
    // 3. SERVICES
    // =========================================================================
    let anime_service = Arc::new(AnimeService::new(
        anime_repo.clone(),
        anime_alias_repo.clone(),
        external_ref_repo.clone(),
        event_bus.clone(),
    ));
    let episode_service = Arc::new(EpisodeService::new(
        episode_repo.clone(),
        anime_repo.clone(),
        file_repo.clone(),
        event_bus.clone(),
    ));
    let file_service = Arc::new(FileService::new(file_repo.clone(), event_bus.clone()));
    let playback_service = Arc::new(PlaybackService::new(
        episode_repo.clone(),
        file_repo.clone(),
        event_bus.clone(),
        mpv_client.clone(),
    ));

    // Register event handlers (same as main.rs)
    episode_service.register_event_handlers();

    println!("[SETUP] Services initialized.");
    println!();

    // =========================================================================
    // 4. GET TEST VIDEO FILE FROM ARGS
    // =========================================================================
    let video_path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            eprintln!("Usage: cargo run --example playback_e2e_test -- <video_file_path>");
            eprintln!("       The file should be a valid video file (e.g., .mkv, .mp4).");
            std::process::exit(1);
        });

    if !video_path.exists() {
        eprintln!("[ERROR] Video file does not exist: {:?}", video_path);
        std::process::exit(1);
    }

    if !video_path.is_file() {
        eprintln!("[ERROR] Path is not a file: {:?}", video_path);
        std::process::exit(1);
    }

    println!("[TEST] Video file: {:?}", video_path);
    println!();

    // =========================================================================
    // 5. STEP 1: SCAN DIRECTORY (to register the file)
    // =========================================================================
    println!("[STEP 1] Scanning directory to register file...");

    let parent_dir = video_path
        .parent()
        .ok_or("Cannot get parent directory")?
        .to_path_buf();

    let files_found = file_service.scan_directory(parent_dir)?;
    println!("[STEP 1] Files found: {}", files_found);

    // Retrieve the file entity by path
    let file = file_repo
        .get_by_path(&video_path.to_string_lossy())?
        .ok_or("File was not registered after scan")?;

    if file.tipo != FileType::Video {
        eprintln!("[ERROR] File is not a video type: {:?}", file.tipo);
        std::process::exit(1);
    }

    println!("[STEP 1] File registered with ID: {}", file.id);
    println!();

    // =========================================================================
    // 6. STEP 2: CREATE ANIME (via service)
    // =========================================================================
    println!("[STEP 2] Creating Anime via AnimeService...");

    let create_anime_request = CreateAnimeRequest {
        titulo_principal: "Test Anime for Playback E2E".to_string(),
        titulos_alternativos: vec![],
        tipo: AnimeType::TV,
        status: AnimeStatus::EmExibicao,
        total_episodios: Some(12),
        data_inicio: None,
        data_fim: None,
        metadados_livres: serde_json::json!({}),
    };

    let anime_id = anime_service.create_anime(create_anime_request)?;
    println!("[STEP 2] Anime created with ID: {}", anime_id);
    println!();

    // =========================================================================
    // 7. STEP 3: CREATE EPISODE (via service)
    // =========================================================================
    println!("[STEP 3] Creating Episode via EpisodeService...");

    let create_episode_request = CreateEpisodeRequest {
        anime_id,
        numero: EpisodeNumber::Regular { numero: 1 },
        titulo: Some("Episode 1 - Test".to_string()),
        duracao_esperada: Some(1440), // 24 minutes in seconds
    };

    let episode_id = episode_service.create_episode(create_episode_request)?;
    println!("[STEP 3] Episode created with ID: {}", episode_id);
    println!();

    // =========================================================================
    // 8. STEP 4: LINK FILE TO EPISODE (via service)
    // =========================================================================
    println!("[STEP 4] Linking File to Episode via EpisodeService...");

    // CORRECTED: LinkFileRequest no longer has is_primary field
    // is_primary is determined from file type in the service
    let link_request = LinkFileRequest {
        episode_id,
        file_id: file.id,
    };

    episode_service.link_file(link_request)?;
    println!("[STEP 4] File linked to Episode as primary.");
    println!();

    // =========================================================================
    // 9. STEP 5: START PLAYBACK (via service)
    // =========================================================================
    println!("[STEP 5] Starting playback via PlaybackService...");
    println!("--- Events will be printed below ---");
    println!();

    // CORRECTED: file_id is now required (not Option)
    let playback_request = StartPlaybackRequest {
        episode_id,
        file_id: file.id,
    };

    let played_path = playback_service.start_playback(playback_request)?;
    println!();
    println!("[STEP 5] Playback started for: {:?}", played_path);
    println!();

    // =========================================================================
    // 10. STEP 6: LET PLAYBACK RUN BRIEFLY
    // =========================================================================
    println!("[STEP 6] Letting playback run for 5 seconds...");
    println!("         (MPV window should be visible)");
    sleep(Duration::from_secs(5));
    println!("[STEP 6] Wait complete.");
    println!();

    // =========================================================================
    // 11. STEP 7: STOP PLAYBACK (via service)
    // =========================================================================
    println!("[STEP 7] Stopping playback via PlaybackService...");

    playback_service.stop_playback(episode_id)?;

    println!("[STEP 7] Playback stopped.");
    println!();

    // Give MPV time to close
    sleep(Duration::from_millis(500));

    // =========================================================================
    // 12. FINAL RESULT
    // =========================================================================
    println!("===========================================");
    println!("PLAYBACK E2E TEST: PASSED");
    println!("===========================================");
    println!();
    println!("Summary:");
    println!("  - Video file: {:?}", video_path);
    println!("  - File ID: {}", file.id);
    println!("  - Anime ID: {}", anime_id);
    println!("  - Episode ID: {}", episode_id);
    println!("  - Playback started: YES");
    println!("  - Playback stopped: YES");
    println!("  - MPV opened: YES (verify visually)");
    println!("  - MPV closed: YES (verify visually)");
    println!();
    println!("This test validates the post-resolution playback flow.");

    Ok(())
}
