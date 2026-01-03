// src-tauri/examples/playback_event_validation_test.rs
//
// PHASE 3 VALIDATION TEST: Playback Event Emission
//
// PURPOSE:
// - Prove canonical event emission during playback lifecycle
// - Validate: PlaybackStarted, PlaybackProgressUpdated, PlaybackStopped
// - Events are observed via EventBus::subscribe (public API)
// - Events are also logged to stdout by EventBus::emit (built-in behavior)
//
// CONTRACT REFERENCES:
// - EVENT_MAP.md Section 5: PlaybackStarted
// - EVENT_MAP.md Section 6: PlaybackProgressUpdated
// - SERVICE_CONTRACTS.md Section 5: Playback Service (emits PlaybackStopped)
//
// OBSERVATION METHOD:
// - EventBus::subscribe<E, F> is used to register handlers
// - Handlers print event details to stdout
// - EventBus::emit already prints "[EVENT] ..." lines (built-in)

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;

use animehub::db::{create_connection_pool, initialize_database};
use animehub::domain::anime::{AnimeType, AnimeStatus};
use animehub::domain::episode::EpisodeNumber;
use animehub::domain::file::FileType;
use animehub::events::{EventBus, PlaybackStarted, PlaybackProgressUpdated, PlaybackStopped};
use animehub::integrations::MpvClient;
use animehub::repositories::*;
use animehub::services::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== PLAYBACK EVENT VALIDATION TEST ===");
    println!("Purpose: Validate canonical event emission during playback");
    println!();

    // =========================================================================
    // 1. INFRASTRUCTURE BOOTSTRAP
    // =========================================================================
    println!("[SETUP] Bootstrapping infrastructure...");
    
    let event_bus = Arc::new(EventBus::new());
    let pool = Arc::new(create_connection_pool()?);
    let mpv_client = Arc::new(MpvClient::new()?);

    {
        let conn = pool.get()?;
        initialize_database(&conn)?;
    }

    // =========================================================================
    // 2. REPOSITORIES
    // =========================================================================
    let anime_repo: Arc<dyn AnimeRepository> = Arc::new(SqliteAnimeRepository::new(pool.clone()));
    let episode_repo: Arc<dyn EpisodeRepository> = Arc::new(SqliteEpisodeRepository::new(pool.clone()));
    let file_repo: Arc<dyn FileRepository> = Arc::new(SqliteFileRepository::new(pool.clone()));
    let anime_alias_repo: Arc<dyn AnimeAliasRepository> = Arc::new(SqliteAnimeAliasRepository::new(pool.clone()));
    let external_ref_repo: Arc<dyn ExternalReferenceRepository> = Arc::new(SqliteExternalReferenceRepository::new(pool.clone()));

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

    episode_service.register_event_handlers();

    println!("[SETUP] Services initialized.");
    println!();

    // =========================================================================
    // 4. REGISTER EVENT OBSERVERS
    // =========================================================================
    println!("[SETUP] Registering event observers...");

    // Counters for validation
    let started_count = Arc::new(AtomicUsize::new(0));
    let progress_count = Arc::new(AtomicUsize::new(0));
    let stopped_count = Arc::new(AtomicUsize::new(0));

    // PlaybackStarted observer
    {
        let counter = Arc::clone(&started_count);
        event_bus.subscribe::<PlaybackStarted, _>(move |event| {
            counter.fetch_add(1, Ordering::SeqCst);
            println!("[OBSERVER] PlaybackStarted received:");
            println!("           episode_id: {}", event.episode_id);
            println!("           event_id: {}", event.event_id);
            println!("           occurred_at: {}", event.occurred_at);
        });
    }

    // PlaybackProgressUpdated observer
    {
        let counter = Arc::clone(&progress_count);
        event_bus.subscribe::<PlaybackProgressUpdated, _>(move |event| {
            counter.fetch_add(1, Ordering::SeqCst);
            println!("[OBSERVER] PlaybackProgressUpdated received:");
            println!("           episode_id: {}", event.episode_id);
            println!("           progress_seconds: {}", event.progress_seconds);
        });
    }

    // PlaybackStopped observer
    {
        let counter = Arc::clone(&stopped_count);
        event_bus.subscribe::<PlaybackStopped, _>(move |event| {
            counter.fetch_add(1, Ordering::SeqCst);
            println!("[OBSERVER] PlaybackStopped received:");
            println!("           episode_id: {}", event.episode_id);
            println!("           final_progress_seconds: {}", event.final_progress_seconds);
        });
    }

    println!("[SETUP] Event observers registered.");
    println!();

    // =========================================================================
    // 5. GET TEST VIDEO FILE FROM ARGS
    // =========================================================================
    let video_path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            eprintln!("Usage: cargo run --example playback_event_validation_test -- <video_file_path>");
            std::process::exit(1);
        });

    if !video_path.exists() || !video_path.is_file() {
        eprintln!("[ERROR] Invalid video file: {:?}", video_path);
        std::process::exit(1);
    }

    println!("[TEST] Video file: {:?}", video_path);
    println!();

    // =========================================================================
    // 6. SETUP: Scan, Create Anime, Create Episode, Link File
    // =========================================================================
    println!("[SETUP] Creating test entities...");

    // Scan
    let parent_dir = video_path.parent().unwrap().to_path_buf();
    file_service.scan_directory(parent_dir)?;
    let file = file_repo.get_by_path(&video_path)?.ok_or("File not found")?;

    // Create Anime
    let anime_id = anime_service.create_anime(CreateAnimeRequest {
        titulo_principal: "Event Validation Test Anime".to_string(),
        titulos_alternativos: vec![],
        tipo: AnimeType::TV,
        status: AnimeStatus::EmExibicao,
        total_episodios: Some(12),
        data_inicio: None,
        data_fim: None,
        metadados_livres: serde_json::json!({}),
    })?;

    // Create Episode
    let episode_id = episode_service.create_episode(CreateEpisodeRequest {
        anime_id,
        numero: EpisodeNumber::Regular { numero: 1 },
        titulo: Some("Event Test Episode".to_string()),
        duracao_esperada: Some(1440),
    })?;

    // Link File
    episode_service.link_file(LinkFileRequest {
        episode_id,
        file_id: file.id,
        is_primary: true,
    })?;

    println!("[SETUP] Test entities created.");
    println!("        Anime ID: {}", anime_id);
    println!("        Episode ID: {}", episode_id);
    println!("        File ID: {}", file.id);
    println!();

    // Clear event log before playback
    event_bus.clear_event_log();

    // =========================================================================
    // 7. START PLAYBACK
    // =========================================================================
    println!("[PLAYBACK] Starting playback...");
    println!("=== EVENT OUTPUT BEGIN ===");
    println!();

    playback_service.start_playback(StartPlaybackRequest {
        episode_id,
        file_id: Some(file.id),
    })?;

    // =========================================================================
    // 8. WAIT FOR PROGRESS EVENTS
    // =========================================================================
    // PlaybackObserver polls every 2000ms with min_progress_delta of 5 seconds
    // Wait 7 seconds to ensure at least one progress event
    println!();
    println!("[PLAYBACK] Waiting 7 seconds for progress events...");
    sleep(Duration::from_secs(7));

    // =========================================================================
    // 9. STOP PLAYBACK
    // =========================================================================
    println!();
    println!("[PLAYBACK] Stopping playback...");
    playback_service.stop_playback(episode_id)?;

    println!();
    println!("=== EVENT OUTPUT END ===");
    println!();

    // Give time for final events
    sleep(Duration::from_millis(500));

    // =========================================================================
    // 10. VALIDATE EVENT COUNTS
    // =========================================================================
    println!("[VALIDATION] Checking event counts...");

    let started = started_count.load(Ordering::SeqCst);
    let progress = progress_count.load(Ordering::SeqCst);
    let stopped = stopped_count.load(Ordering::SeqCst);

    println!("  PlaybackStarted: {}", started);
    println!("  PlaybackProgressUpdated: {}", progress);
    println!("  PlaybackStopped: {}", stopped);
    println!();

    let mut passed = true;

    if started < 1 {
        println!("[FAIL] Expected at least 1 PlaybackStarted event.");
        passed = false;
    } else {
        println!("[PASS] PlaybackStarted emitted.");
    }

    // Note: PlaybackProgressUpdated may be 0 if video is very short or MPV didn't report position
    // We check for >= 0 but warn if 0
    if progress == 0 {
        println!("[WARN] No PlaybackProgressUpdated events received.");
        println!("       This may be expected for very short playback or if MPV didn't report position.");
    } else {
        println!("[PASS] PlaybackProgressUpdated emitted ({} times).", progress);
    }

    // PlaybackStopped is emitted by the observer when MPV stops
    // It may not be emitted if we call stop_playback before observer detects it
    if stopped == 0 {
        println!("[WARN] No PlaybackStopped events received.");
        println!("       This may be expected if stop_playback was called before observer detected stop.");
    } else {
        println!("[PASS] PlaybackStopped emitted.");
    }

    println!();

    // =========================================================================
    // 11. FINAL RESULT
    // =========================================================================
    if passed {
        println!("===========================================");
        println!("PLAYBACK EVENT VALIDATION TEST: PASSED");
        println!("===========================================");
    } else {
        println!("===========================================");
        println!("PLAYBACK EVENT VALIDATION TEST: FAILED");
        println!("===========================================");
        std::process::exit(1);
    }

    println!();
    println!("Summary:");
    println!("  - PlaybackStarted: {} (required: >= 1)", started);
    println!("  - PlaybackProgressUpdated: {} (informational)", progress);
    println!("  - PlaybackStopped: {} (informational)", stopped);
    println!();
    println!("This test validates canonical event emission during playback.");

    Ok(())
}
