// src-tauri/src/application/state.rs
// CORRECTED VERSION

use std::sync::Arc;

use crate::events::EventBus;
use crate::services::{
    AnimeService, EpisodeService, FileService, PlaybackService,
    StatisticsService, ExternalIntegrationService, SubtitleService,
};

/// Application state managed by Tauri.
/// All fields are Arc-wrapped for thread-safe sharing across commands.
/// Services are initialized in main.rs and passed here.
pub struct AppState {
    pub event_bus: Arc<EventBus>,
    pub anime_service: Arc<AnimeService>,
    pub episode_service: Arc<EpisodeService>,
    pub file_service: Arc<FileService>,
    pub playback_service: Arc<PlaybackService>,
    pub statistics_service: Arc<StatisticsService>,
    pub external_integration_service: Arc<ExternalIntegrationService>,
    pub subtitle_service: Arc<SubtitleService>,
}
