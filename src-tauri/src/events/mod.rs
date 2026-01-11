// src-tauri/src/events/mod.rs
//
// Internal Event System - Public API
//
// CRITICAL: EventHandler is INTERNAL and must NOT be exported

// ============================================================================
// EXISTING EVENT INFRASTRUCTURE (SEALED - Phase 3)
// ============================================================================

pub mod bus;
pub mod types;

// ============================================================================
// RESOLUTION EVENTS (FROZEN - Phase 4)
// ============================================================================

pub mod resolution_events;

// ============================================================================
// MATERIALIZATION (NEW - Phase 5)
// ============================================================================

pub mod handlers;
pub mod materialization_events;

// ============================================================================
// PUBLIC EXPORTS - Event Types and Bus Only
// ============================================================================

pub use types::DomainEvent;

pub use types::{
    // Anime
    AnimeCreated,
    AnimeMerged,

    AnimeUpdated,
    // File scanning
    DirectoryScanned,
    EpisodeBecamePlayable,
    EpisodeCompleted,

    // Episode
    EpisodeCreated,
    EpisodeProgressUpdated,
    ExternalMetadataFetched,
    ExternalMetadataLinked,
    // External
    ExternalMetadataRequested,
    FileDetected,

    FileLinkedToEpisode,
    PlaybackFinished,

    PlaybackProgressUpdated,
    // Playback
    PlaybackStarted,
    PlaybackStopped,
    // Statistics
    StatisticsRebuilt,
    StatisticsUpdated,

    // Subtitle
    SubtitleDetected,
    SubtitleStyleApplied,
    SubtitleTimingAdjusted,
    SubtitleVersionCreated,
};

pub use bus::{EventBus, EventLogEntry};

// Resolution events (Phase 4 - frozen)
pub use resolution_events::{
    EpisodeResolved, FileResolved, ResolutionBatchCompleted, ResolutionFailed,
};

// Materialization events (Phase 5)
pub use materialization_events::{MaterializationBatchCompleted, MaterializationRecordCreated};

// Materialization handler registration (Phase 5)
pub use handlers::register_materialization_handlers;

// ============================================================================
// INTERNAL ONLY - DO NOT EXPORT
// ============================================================================

// EventHandler type alias is internal to the bus module
// handlers::RegisterHandler is internal to handlers module

/// Initialize a new event bus
pub fn create_event_bus() -> EventBus {
    EventBus::new()
}
