// src-tauri/src/services/mod.rs
//
// Services Module - Orchestration Layer

pub mod anime_service;
pub mod episode_service;
pub mod file_service;
pub mod playback_service;
pub mod statistics_service;
pub mod external_integration_service;
pub mod subtitle_service;
pub mod playback_observer;
pub mod resolution_service;

#[cfg(test)]
mod resolution_service_tests;

// Re-export all services and their types
pub use anime_service::{
    AnimeService,
    CreateAnimeRequest,
    UpdateAnimeRequest,
    MergeAnimesRequest,
};

pub use episode_service::{
    EpisodeService,
    CreateEpisodeRequest,
    UpdateEpisodeMetadataRequest,
    LinkFileRequest,
};

pub use file_service::{
    FileService,
    RegisterFileRequest,
};

pub use playback_service::{
    PlaybackService,
    StartPlaybackRequest,
};

pub use statistics_service::{
    StatisticsService,
};

pub use external_integration_service::{
    ExternalIntegrationService,
    FetchMetadataRequest,
    LinkExternalReferenceRequest,
    ExternalMetadata,
    MetadataSuggestions,
};

pub use subtitle_service::{
    SubtitleService,
    StyleTransformRequest,
    TimingTransformRequest,
};

pub use resolution_service::{
    ResolutionService,
    ResolutionRules,
};
