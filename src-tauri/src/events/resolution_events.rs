// src-tauri/src/events/resolution_events.rs
//
// Resolution Events - Phase 4
//
// Events that represent knowledge derived from resolution, not actions.
// These events are the bridge between raw scan data and domain mutation.
//
// CRITICAL RULES:
// - Events are facts, not commands
// - Events are immutable
// - Events carry only the data needed to react
// - No business logic in event types
// - Resolution events represent knowledge, not action

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

use super::types::DomainEvent;

// ============================================================================
// FILE RESOLVED EVENT
// ============================================================================

/// Emitted when a file is successfully resolved to domain intent.
/// This represents knowledge about what the file is.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileResolved {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    
    /// The file ID that was resolved
    pub file_id: Uuid,
    
    /// The file path (for traceability)
    pub file_path: PathBuf,
    
    /// The resolved anime title
    pub anime_title: String,
    
    /// If matched to existing anime, its ID
    pub matched_anime_id: Option<Uuid>,
    
    /// The resolved episode number (as string for flexibility)
    pub episode_number: String,
    
    /// If matched to existing episode, its ID
    pub matched_episode_id: Option<Uuid>,
    
    /// The role of the file (video, subtitle, etc.)
    pub file_role: String,
    
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
    
    /// Source of resolution (filename, folder, etc.)
    pub resolution_source: String,
}

impl FileResolved {
    pub fn new(
        file_id: Uuid,
        file_path: PathBuf,
        anime_title: String,
        matched_anime_id: Option<Uuid>,
        episode_number: String,
        matched_episode_id: Option<Uuid>,
        file_role: String,
        confidence: f64,
        resolution_source: String,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            file_id,
            file_path,
            anime_title,
            matched_anime_id,
            episode_number,
            matched_episode_id,
            file_role,
            confidence,
            resolution_source,
        }
    }
}

impl DomainEvent for FileResolved {
    fn event_id(&self) -> Uuid { self.event_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn event_type(&self) -> &'static str { "FileResolved" }
}

// ============================================================================
// EPISODE RESOLVED EVENT
// ============================================================================

/// Emitted when an episode is resolved from one or more files.
/// This aggregates file resolutions into episode-level knowledge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeResolved {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    
    /// The resolved anime title
    pub anime_title: String,
    
    /// If matched to existing anime, its ID
    pub matched_anime_id: Option<Uuid>,
    
    /// The resolved episode number
    pub episode_number: String,
    
    /// If matched to existing episode, its ID
    pub matched_episode_id: Option<Uuid>,
    
    /// The primary video file ID (if found)
    pub video_file_id: Option<Uuid>,
    
    /// Associated subtitle file IDs
    pub subtitle_file_ids: Vec<Uuid>,
    
    /// Overall confidence for this episode resolution
    pub confidence: f64,
}

impl EpisodeResolved {
    pub fn new(
        anime_title: String,
        matched_anime_id: Option<Uuid>,
        episode_number: String,
        matched_episode_id: Option<Uuid>,
        video_file_id: Option<Uuid>,
        subtitle_file_ids: Vec<Uuid>,
        confidence: f64,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            anime_title,
            matched_anime_id,
            episode_number,
            matched_episode_id,
            video_file_id,
            subtitle_file_ids,
            confidence,
        }
    }
}

impl DomainEvent for EpisodeResolved {
    fn event_id(&self) -> Uuid { self.event_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn event_type(&self) -> &'static str { "EpisodeResolved" }
}

// ============================================================================
// RESOLUTION FAILED EVENT
// ============================================================================

/// Emitted when resolution fails for a file.
/// This is explicit, structured, and non-fatal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionFailed {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    
    /// The file ID that failed to resolve
    pub file_id: Uuid,
    
    /// The file path (for traceability)
    pub file_path: PathBuf,
    
    /// The reason for failure
    pub failure_reason: String,
    
    /// Human-readable description
    pub description: String,
}

impl ResolutionFailed {
    pub fn new(
        file_id: Uuid,
        file_path: PathBuf,
        failure_reason: String,
        description: String,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            file_id,
            file_path,
            failure_reason,
            description,
        }
    }
}

impl DomainEvent for ResolutionFailed {
    fn event_id(&self) -> Uuid { self.event_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn event_type(&self) -> &'static str { "ResolutionFailed" }
}

// ============================================================================
// RESOLUTION BATCH COMPLETED EVENT
// ============================================================================

/// Emitted when a batch resolution operation completes.
/// Provides summary statistics for the batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionBatchCompleted {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    
    /// Total files processed
    pub total_files: usize,
    
    /// Successfully resolved files
    pub resolved_count: usize,
    
    /// Failed resolutions
    pub failed_count: usize,
    
    /// Skipped files (already resolved or not applicable)
    pub skipped_count: usize,
    
    /// Duration of the batch operation in milliseconds
    pub duration_ms: u64,
}

impl ResolutionBatchCompleted {
    pub fn new(
        total_files: usize,
        resolved_count: usize,
        failed_count: usize,
        skipped_count: usize,
        duration_ms: u64,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            total_files,
            resolved_count,
            failed_count,
            skipped_count,
            duration_ms,
        }
    }
}

impl DomainEvent for ResolutionBatchCompleted {
    fn event_id(&self) -> Uuid { self.event_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn event_type(&self) -> &'static str { "ResolutionBatchCompleted" }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_resolved_event_creation() {
        let event = FileResolved::new(
            Uuid::new_v4(),
            PathBuf::from("/anime/Steins Gate/Episode 01.mkv"),
            "Steins;Gate".to_string(),
            None,
            "1".to_string(),
            None,
            "video".to_string(),
            0.95,
            "filename".to_string(),
        );
        
        assert_eq!(event.event_type(), "FileResolved");
        assert_eq!(event.anime_title, "Steins;Gate");
        assert_eq!(event.episode_number, "1");
        assert_eq!(event.confidence, 0.95);
    }

    #[test]
    fn test_episode_resolved_event_creation() {
        let video_id = Uuid::new_v4();
        let sub_id = Uuid::new_v4();
        
        let event = EpisodeResolved::new(
            "Steins;Gate".to_string(),
            None,
            "1".to_string(),
            None,
            Some(video_id),
            vec![sub_id],
            0.90,
        );
        
        assert_eq!(event.event_type(), "EpisodeResolved");
        assert_eq!(event.video_file_id, Some(video_id));
        assert_eq!(event.subtitle_file_ids.len(), 1);
    }

    #[test]
    fn test_resolution_failed_event_creation() {
        let event = ResolutionFailed::new(
            Uuid::new_v4(),
            PathBuf::from("/unknown/file.bin"),
            "unsupported_file_type".to_string(),
            "Binary files are not supported for resolution".to_string(),
        );
        
        assert_eq!(event.event_type(), "ResolutionFailed");
        assert_eq!(event.failure_reason, "unsupported_file_type");
    }

    #[test]
    fn test_resolution_batch_completed_event() {
        let event = ResolutionBatchCompleted::new(100, 85, 10, 5, 1500);
        
        assert_eq!(event.event_type(), "ResolutionBatchCompleted");
        assert_eq!(event.total_files, 100);
        assert_eq!(event.resolved_count, 85);
        assert_eq!(event.failed_count, 10);
        assert_eq!(event.skipped_count, 5);
    }
}
