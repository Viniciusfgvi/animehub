// src-tauri/src/main.rs
// CORRECTED VERSION

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::Arc;

// --- Corrected Imports ---
// Direct imports for Tauri command handler macro
use animehub::application::commands::*;
// Correct path for database module
use animehub::db::{create_connection_pool, initialize_database};
// All other necessary components for initialization
use animehub::application::state::AppState;
use animehub::events::EventBus;
use animehub::integrations::MpvClient;
use animehub::repositories::*;
use animehub::services::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. INFRASTRUCTURE
    let event_bus = Arc::new(EventBus::new());
    let pool = Arc::new(create_connection_pool()?);
    let mpv_client = Arc::new(MpvClient::new()?);
    
    // Initialize schema (idempotent)
    {
        let conn = pool.get()?;
        initialize_database(&conn)?;
    }

    // 2. REPOSITORIES
    // The type `Arc<dyn Trait>` is used to match the service constructor signatures exactly.
    let anime_repo: Arc<dyn AnimeRepository> = Arc::new(SqliteAnimeRepository::new(pool.clone()));
    let episode_repo: Arc<dyn EpisodeRepository> = Arc::new(SqliteEpisodeRepository::new(pool.clone()));
    let file_repo: Arc<dyn FileRepository> = Arc::new(SqliteFileRepository::new(pool.clone()));
    let subtitle_repo: Arc<dyn SubtitleRepository> = Arc::new(SqliteSubtitleRepository::new(pool.clone()));
    let collection_repo: Arc<dyn CollectionRepository> = Arc::new(SqliteCollectionRepository::new(pool.clone()));
    let external_ref_repo: Arc<dyn ExternalReferenceRepository> = Arc::new(SqliteExternalReferenceRepository::new(pool.clone()));
    let anime_alias_repo: Arc<dyn AnimeAliasRepository> = Arc::new(SqliteAnimeAliasRepository::new(pool.clone()));
    let statistics_repo: Arc<dyn StatisticsRepository> = Arc::new(SqliteStatisticsRepository::new(pool.clone()));

    // 3. SERVICES
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
    let statistics_service = Arc::new(StatisticsService::new(
        statistics_repo.clone(),
        anime_repo.clone(),
        episode_repo.clone(),
        event_bus.clone(),
    ));
    let external_integration_service = Arc::new(ExternalIntegrationService::new(
        external_ref_repo.clone(),
        anime_repo.clone(),
        event_bus.clone(),
    ));
    let subtitle_service = Arc::new(SubtitleService::new(
        subtitle_repo.clone(),
        file_repo.clone(),
        event_bus.clone(),
    ));

    // 4. EVENT HANDLER REGISTRATION (WIRING)
    episode_service.register_event_handlers();
    statistics_service.register_event_handlers();

    // 5. APPLICATION STATE
    let app_state = AppState {
        event_bus,
        anime_service,
        episode_service,
        file_service,
        playback_service,
        statistics_service,
        external_integration_service,
        subtitle_service,
    };

    // 6. TAURI BOOTSTRAP
    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            // Commands are now in scope via `use` statements
            list_animes,
            get_anime,
            create_anime,
            update_anime,
            list_episodes,
            get_episode,
            create_episode,
            update_progress,
            mark_episode_completed,
            reset_episode_progress,
            scan_directory,
            get_episode_files,
            start_playback,
            toggle_pause_playback,
            seek_playback,
            stop_playback,
            get_episode_progress,
            get_global_statistics,
        ])
        .run(tauri::generate_context!())?;

    Ok(())
}
