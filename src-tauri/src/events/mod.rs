// src-tauri/src/events/mod.rs
//
// Internal Event System - Public API
//
// CRITICAL: EventHandler is INTERNAL and must NOT be exported

pub mod types;
pub mod bus;
pub mod handlers;

// ============================================================================
// PUBLIC EXPORTS - Event Types and Bus Only
// ============================================================================

pub use types::{
    DomainEvent,
    
    // File scanning
    DirectoryScanned,
    FileDetected,
    
    // Anime
    AnimeCreated,
    AnimeUpdated,
    AnimeMerged,
    
    // Episode
    EpisodeCreated,
    FileLinkedToEpisode,
    EpisodeBecamePlayable,
    EpisodeProgressUpdated,
    EpisodeCompleted,
    
    // Playback
    PlaybackStarted,
    PlaybackProgressUpdated,
    PlaybackStopped,
    
    // Subtitle
    SubtitleDetected,
    SubtitleStyleApplied,
    SubtitleTimingAdjusted,
    SubtitleVersionCreated,
    
    // Statistics
    StatisticsRebuilt,
    StatisticsUpdated,
    
    // External
    ExternalMetadataRequested,
    ExternalMetadataFetched,
    ExternalMetadataLinked,
};

pub use bus::{EventBus, EventLogEntry};

// ============================================================================
// INTERNAL ONLY - DO NOT EXPORT
// ============================================================================

// EventHandler type alias is internal to the bus module
// handlers::RegisterHandler is internal to handlers module

/// Initialize a new event bus
pub fn create_event_bus() -> EventBus {
    EventBus::new()
}