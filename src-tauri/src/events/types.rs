// events/types.rs
//
// All domain events in the system.
// Each event represents an immutable fact that has already occurred.
//
// CRITICAL RULES:
// - Events are facts, not commands
// - Events are immutable
// - Events carry only the data needed to react
// - No business logic in event types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::path::PathBuf;

/// Trait that all domain events must implement
pub trait DomainEvent: std::fmt::Debug + Clone {
    /// Unique identifier for this event instance
    fn event_id(&self) -> Uuid;
    
    /// When this event occurred
    fn occurred_at(&self) -> DateTime<Utc>;
    
    /// Human-readable event type name
    fn event_type(&self) -> &'static str;
}

// ============================================================================
// FILE SCANNING EVENTS
// ============================================================================

/// Emitted when a directory scan completes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryScanned {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub directory_path: PathBuf,
    pub files_found: usize,
}

impl DirectoryScanned {
    pub fn new(directory_path: PathBuf, files_found: usize) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            directory_path,
            files_found,
        }
    }
}

impl DomainEvent for DirectoryScanned {
    fn event_id(&self) -> Uuid { self.event_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn event_type(&self) -> &'static str { "DirectoryScanned" }
}

/// Emitted for each relevant file detected during scan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDetected {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub file_path: PathBuf,
    pub file_size: u64,
    pub file_type: String, // "video", "subtitle", "image"
}

impl FileDetected {
    pub fn new(file_path: PathBuf, file_size: u64, file_type: String) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            file_path,
            file_size,
            file_type,
        }
    }
}

impl DomainEvent for FileDetected {
    fn event_id(&self) -> Uuid { self.event_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn event_type(&self) -> &'static str { "FileDetected" }
}

// ============================================================================
// ANIME DOMAIN EVENTS
// ============================================================================

/// Emitted when a new Anime entity is created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimeCreated {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub anime_id: Uuid,
    pub titulo_principal: String,
    pub tipo: String, // "TV", "Movie", "OVA", "Special"
}

impl AnimeCreated {
    pub fn new(anime_id: Uuid, titulo_principal: String, tipo: String) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            anime_id,
            titulo_principal,
            tipo,
        }
    }
}

impl DomainEvent for AnimeCreated {
    fn event_id(&self) -> Uuid { self.event_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn event_type(&self) -> &'static str { "AnimeCreated" }
}

/// Emitted when anime metadata is updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimeUpdated {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub anime_id: Uuid,
}

impl AnimeUpdated {
    pub fn new(anime_id: Uuid) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            anime_id,
        }
    }
}

impl DomainEvent for AnimeUpdated {
    fn event_id(&self) -> Uuid { self.event_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn event_type(&self) -> &'static str { "AnimeUpdated" }
}

/// Emitted when duplicate animes are merged
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimeMerged {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub principal_anime_id: Uuid,
    pub merged_anime_id: Uuid,
}

impl AnimeMerged {
    pub fn new(principal_anime_id: Uuid, merged_anime_id: Uuid) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            principal_anime_id,
            merged_anime_id,
        }
    }
}

impl DomainEvent for AnimeMerged {
    fn event_id(&self) -> Uuid { self.event_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn event_type(&self) -> &'static str { "AnimeMerged" }
}

// ============================================================================
// EPISODE DOMAIN EVENTS
// ============================================================================

/// Emitted when a new Episode is created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeCreated {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub episode_id: Uuid,
    pub anime_id: Uuid,
    pub numero: String, // Can be "1", "2" or "OVA 1", etc
}

impl EpisodeCreated {
    pub fn new(episode_id: Uuid, anime_id: Uuid, numero: String) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            episode_id,
            anime_id,
            numero,
        }
    }
}

impl DomainEvent for EpisodeCreated {
    fn event_id(&self) -> Uuid { self.event_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn event_type(&self) -> &'static str { "EpisodeCreated" }
}

/// Emitted when a file is linked to an episode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileLinkedToEpisode {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub episode_id: Uuid,
    pub file_id: Uuid,
    pub is_primary: bool, // true = main video, false = auxiliary
}

impl FileLinkedToEpisode {
    pub fn new(episode_id: Uuid, file_id: Uuid, is_primary: bool) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            episode_id,
            file_id,
            is_primary,
        }
    }
}

impl DomainEvent for FileLinkedToEpisode {
    fn event_id(&self) -> Uuid { self.event_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn event_type(&self) -> &'static str { "FileLinkedToEpisode" }
}

/// Emitted when an episode becomes playable (has valid video file)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeBecamePlayable {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub episode_id: Uuid,
}

impl EpisodeBecamePlayable {
    pub fn new(episode_id: Uuid) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            episode_id,
        }
    }
}

impl DomainEvent for EpisodeBecamePlayable {
    fn event_id(&self) -> Uuid { self.event_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn event_type(&self) -> &'static str { "EpisodeBecamePlayable" }
}

/// Emitted when episode progress is updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeProgressUpdated {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub episode_id: Uuid,
    pub progress_seconds: u64,
    pub duration_seconds: Option<u64>,
}

impl EpisodeProgressUpdated {
    pub fn new(episode_id: Uuid, progress_seconds: u64, duration_seconds: Option<u64>) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            episode_id,
            progress_seconds,
            duration_seconds,
        }
    }
}

impl DomainEvent for EpisodeProgressUpdated {
    fn event_id(&self) -> Uuid { self.event_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn event_type(&self) -> &'static str { "EpisodeProgressUpdated" }
}

/// Emitted when an episode is marked as completed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeCompleted {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub episode_id: Uuid,
    pub anime_id: Uuid,
}

impl EpisodeCompleted {
    pub fn new(episode_id: Uuid, anime_id: Uuid) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            episode_id,
            anime_id,
        }
    }
}

impl DomainEvent for EpisodeCompleted {
    fn event_id(&self) -> Uuid { self.event_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn event_type(&self) -> &'static str { "EpisodeCompleted" }
}

// ============================================================================
// PLAYBACK EVENTS
// ============================================================================

/// Emitted when playback starts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackStarted {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub episode_id: Uuid,
}

impl PlaybackStarted {
    pub fn new(episode_id: Uuid) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            episode_id,
        }
    }
}

impl DomainEvent for PlaybackStarted {
    fn event_id(&self) -> Uuid { self.event_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn event_type(&self) -> &'static str { "PlaybackStarted" }
}

/// Emitted periodically during playback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackProgressUpdated {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub episode_id: Uuid,
    pub progress_seconds: u64,
}

impl PlaybackProgressUpdated {
    pub fn new(episode_id: Uuid, progress_seconds: u64) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            episode_id,
            progress_seconds,
        }
    }
}

impl DomainEvent for PlaybackProgressUpdated {
    fn event_id(&self) -> Uuid { self.event_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn event_type(&self) -> &'static str { "PlaybackProgressUpdated" }
}

/// Emitted when playback stops
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackStopped {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub episode_id: Uuid,
    pub final_progress_seconds: u64,
}

impl PlaybackStopped {
    pub fn new(episode_id: Uuid, final_progress_seconds: u64) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            episode_id,
            final_progress_seconds,
        }
    }
}

impl DomainEvent for PlaybackStopped {
    fn event_id(&self) -> Uuid { self.event_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn event_type(&self) -> &'static str { "PlaybackStopped" }
}

/// Emitted when playback is paused
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackPaused {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub episode_id: Uuid,
    pub position_seconds: u64,
}

impl PlaybackPaused {
    pub fn new(episode_id: Uuid, position_seconds: u64) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            episode_id,
            position_seconds,
        }
    }
}

impl DomainEvent for PlaybackPaused {
    fn event_id(&self) -> Uuid { self.event_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn event_type(&self) -> &'static str { "PlaybackPaused" }
}

/// Emitted when playback is resumed after pause
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackResumed {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub episode_id: Uuid,
}

impl PlaybackResumed {
    pub fn new(episode_id: Uuid) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            episode_id,
        }
    }
}

impl DomainEvent for PlaybackResumed {
    fn event_id(&self) -> Uuid { self.event_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn event_type(&self) -> &'static str { "PlaybackResumed" }
}

/// Emitted when playback finishes naturally (reached end)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackFinished {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub episode_id: Uuid,
    pub duration_seconds: u64,
}

impl PlaybackFinished {
    pub fn new(episode_id: Uuid, duration_seconds: u64) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            episode_id,
            duration_seconds,
        }
    }
}

impl DomainEvent for PlaybackFinished {
    fn event_id(&self) -> Uuid { self.event_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn event_type(&self) -> &'static str { "PlaybackFinished" }
}

// ============================================================================
// SUBTITLE EVENTS
// ============================================================================

/// Emitted when a subtitle file is detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleDetected {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub file_id: Uuid,
    pub format: String, // "SRT", "ASS", "VTT"
    pub language: String,
}

impl SubtitleDetected {
    pub fn new(file_id: Uuid, format: String, language: String) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            file_id,
            format,
            language,
        }
    }
}

impl DomainEvent for SubtitleDetected {
    fn event_id(&self) -> Uuid { self.event_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn event_type(&self) -> &'static str { "SubtitleDetected" }
}

/// Emitted when subtitle style is applied
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleStyleApplied {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub subtitle_id: Uuid,
    pub new_subtitle_id: Uuid, // The derived subtitle
}

impl SubtitleStyleApplied {
    pub fn new(subtitle_id: Uuid, new_subtitle_id: Uuid) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            subtitle_id,
            new_subtitle_id,
        }
    }
}

impl DomainEvent for SubtitleStyleApplied {
    fn event_id(&self) -> Uuid { self.event_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn event_type(&self) -> &'static str { "SubtitleStyleApplied" }
}

/// Emitted when subtitle timing is adjusted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleTimingAdjusted {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub subtitle_id: Uuid,
    pub new_subtitle_id: Uuid, // The derived subtitle
    pub offset_ms: i64,
}

impl SubtitleTimingAdjusted {
    pub fn new(subtitle_id: Uuid, new_subtitle_id: Uuid, offset_ms: i64) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            subtitle_id,
            new_subtitle_id,
            offset_ms,
        }
    }
}

impl DomainEvent for SubtitleTimingAdjusted {
    fn event_id(&self) -> Uuid { self.event_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn event_type(&self) -> &'static str { "SubtitleTimingAdjusted" }
}

/// Emitted when a new subtitle version is created from transformation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleVersionCreated {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub subtitle_id: Uuid,
    pub version: u32,
}

impl SubtitleVersionCreated {
    pub fn new(subtitle_id: Uuid, version: u32) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            subtitle_id,
            version,
        }
    }
}

impl DomainEvent for SubtitleVersionCreated {
    fn event_id(&self) -> Uuid { self.event_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn event_type(&self) -> &'static str { "SubtitleVersionCreated" }
}

// ============================================================================
// STATISTICS EVENTS
// ============================================================================

/// Emitted when statistics are recalculated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticsRebuilt {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub statistics_type: String, // "global", "per_anime", "per_period"
}

impl StatisticsRebuilt {
    pub fn new(statistics_type: String) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            statistics_type,
        }
    }
}

impl DomainEvent for StatisticsRebuilt {
    fn event_id(&self) -> Uuid { self.event_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn event_type(&self) -> &'static str { "StatisticsRebuilt" }
}

/// Emitted when statistics are updated incrementally
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticsUpdated {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
}

impl StatisticsUpdated {
    pub fn new() -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
        }
    }
}

impl DomainEvent for StatisticsUpdated {
    fn event_id(&self) -> Uuid { self.event_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn event_type(&self) -> &'static str { "StatisticsUpdated" }
}

// ============================================================================
// EXTERNAL INTEGRATION EVENTS
// ============================================================================

/// Emitted when external metadata is requested
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalMetadataRequested {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub anime_id: Uuid,
    pub provider: String, // "AniList"
}

impl ExternalMetadataRequested {
    pub fn new(anime_id: Uuid, provider: String) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            anime_id,
            provider,
        }
    }
}

impl DomainEvent for ExternalMetadataRequested {
    fn event_id(&self) -> Uuid { self.event_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn event_type(&self) -> &'static str { "ExternalMetadataRequested" }
}

/// Emitted when external metadata is fetched
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalMetadataFetched {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub anime_id: Uuid,
    pub provider: String,
    pub external_id: String,
}

impl ExternalMetadataFetched {
    pub fn new(anime_id: Uuid, provider: String, external_id: String) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            anime_id,
            provider,
            external_id,
        }
    }
}

impl DomainEvent for ExternalMetadataFetched {
    fn event_id(&self) -> Uuid { self.event_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn event_type(&self) -> &'static str { "ExternalMetadataFetched" }
}

/// Emitted when external metadata is linked to an anime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalMetadataLinked {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub anime_id: Uuid,
    pub provider: String,
    pub external_id: String,
}

impl ExternalMetadataLinked {
    pub fn new(anime_id: Uuid, provider: String, external_id: String) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            anime_id,
            provider,
            external_id,
        }
    }
}

impl DomainEvent for ExternalMetadataLinked {
    fn event_id(&self) -> Uuid { self.event_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn event_type(&self) -> &'static str { "ExternalMetadataLinked" }
}