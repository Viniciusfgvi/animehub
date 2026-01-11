// src-tauri/src/services/mod.rs
//
// Services Module - Orchestration Layer
//
// CRITICAL RULES:
// - Services orchestrate domain operations
// - Services emit events
// - Services enforce invariants
// - Services are the ONLY entry point for mutations

// ============================================================================
// EXISTING SERVICE MODULES (SEALED - Phase 3)
// ============================================================================

pub mod anime_service;
pub mod episode_service;
pub mod external_integration_service;
pub mod file_service;
pub mod playback_observer;
pub mod playback_service;
pub mod statistics_service;
pub mod subtitle_service;

// ============================================================================
// RESOLUTION SERVICE (FROZEN - Phase 4)
// ============================================================================

pub mod resolution_service;

#[cfg(test)]
mod resolution_service_tests;

// ============================================================================
// MATERIALIZATION SERVICE (NEW - Phase 5)
// ============================================================================

pub mod materialization_service;
pub mod materialization_types;

#[cfg(test)]
mod materialization_service_tests;

// ============================================================================
// PUBLIC EXPORTS - Services
// ============================================================================

// Anime Service
pub use anime_service::AnimeService;
pub use anime_service::CreateAnimeRequest;
pub use anime_service::MergeAnimesRequest;
pub use anime_service::UpdateAnimeRequest;

// Episode Service
pub use episode_service::CreateEpisodeRequest;
pub use episode_service::EpisodeService;
pub use episode_service::LinkFileRequest;
pub use episode_service::UpdateEpisodeMetadataRequest;

// File Service
pub use file_service::FileService;
pub use file_service::RegisterFileRequest;

// Playback Service
pub use playback_service::PlaybackService;
pub use playback_service::StartPlaybackRequest;

// Playback Observer
pub use playback_observer::ObserverConfig;
pub use playback_observer::PlaybackObserver;

// Statistics Service
pub use statistics_service::StatisticsService;

// External Integration Service
pub use external_integration_service::ExternalIntegrationService;
pub use external_integration_service::ExternalMetadata;
pub use external_integration_service::FetchMetadataRequest;
pub use external_integration_service::LinkExternalReferenceRequest;
pub use external_integration_service::MetadataSuggestions;

// Subtitle Service
pub use subtitle_service::StyleTransformRequest;
pub use subtitle_service::SubtitleService;
pub use subtitle_service::TimingTransformRequest;

// Resolution Service (Phase 4 - FROZEN)
pub use resolution_service::ResolutionRules;
pub use resolution_service::ResolutionService;

// Materialization Service (Phase 5)
pub use materialization_service::MaterializationService;
pub use materialization_types::EpisodeNumberDecision;
pub use materialization_types::MaterializationDecision;
pub use materialization_types::MaterializationEventType;
pub use materialization_types::MaterializationFingerprint;
pub use materialization_types::MaterializationOutcome;
pub use materialization_types::MaterializationRecord;
pub use materialization_types::MaterializationResult;
