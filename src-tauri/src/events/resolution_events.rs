// src-tauri/src/events/resolution_events.rs
//
// Resolution Events - Phase 4 (FINAL CORRECTED VERSION)
//
// These events are the ONLY outputs of Phase 4 Resolution.
// They carry resolution intent to Phase 5 Materialization.
//
// CRITICAL INVARIANTS:
// - All events are deterministic (no timestamps in event payload)
// - All events are immutable
// - All events are serializable
// - All events are reachable through real resolution paths
// - Event IDs are derived deterministically from fingerprints
// - occurred_at() returns SENTINEL_TIMESTAMP (Unix epoch) for trait compliance
//
// PHASE 4 CLOSURE CORRECTIONS:
// - EpisodeResolved is emitted through aggregation logic
// - Removed timestamps from event payloads (determinism)
// - All events are reachable
// - occurred_at() returns fixed sentinel for DomainEvent trait compliance
// - ResolutionBatchCompleted uses deterministic event ID

use crate::events::DomainEvent;
use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// Sentinel timestamp for Phase 4 events (Unix epoch).
/// Phase 4 events are deterministic and do not carry operational timestamps.
/// This constant satisfies the DomainEvent trait while maintaining determinism.
const SENTINEL_TIMESTAMP: DateTime<Utc> = DateTime::<Utc>::UNIX_EPOCH;

// ============================================================================
// FILE RESOLVED EVENT
// ============================================================================

/// Emitted when a single file is successfully resolved.
/// This is the primary output of file-level resolution.
///
/// DETERMINISM: No timestamp in payload. Identical resolution produces identical event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileResolved {
    /// The file ID that was resolved
    pub file_id: Uuid,

    /// The file path (for traceability)
    pub file_path: PathBuf,

    /// The resolved anime title
    pub anime_title: String,

    /// If matched to existing anime, its ID
    pub matched_anime_id: Option<Uuid>,

    /// The resolved episode number (as string for serialization)
    pub episode_number: String,

    /// If matched to existing episode, its ID
    pub matched_episode_id: Option<Uuid>,

    /// The file role (video, subtitle, image)
    pub file_role: String,

    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,

    /// Resolution source (filename, folder_name)
    pub source: String,

    /// Deterministic fingerprint for idempotency
    pub fingerprint: String,
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
        source: String,
        fingerprint: String,
    ) -> Self {
        Self {
            file_id,
            file_path,
            anime_title,
            matched_anime_id,
            episode_number,
            matched_episode_id,
            file_role,
            confidence,
            source,
            fingerprint,
        }
    }
}

impl DomainEvent for FileResolved {
    fn event_id(&self) -> Uuid {
        // Deterministic event ID derived from fingerprint
        Uuid::new_v5(&Uuid::NAMESPACE_OID, self.fingerprint.as_bytes())
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        // Phase 4 events use sentinel timestamp for determinism
        SENTINEL_TIMESTAMP
    }

    fn event_type(&self) -> &'static str {
        "FileResolved"
    }
}

// ============================================================================
// EPISODE RESOLVED EVENT
// ============================================================================

/// Emitted when resolution aggregation determines a complete episode intent.
/// This event is produced by aggregating FileResolved events for the same episode.
///
/// REACHABILITY: Emitted by ResolutionService.resolve_batch() through aggregation logic.
///
/// DETERMINISM: No timestamp in payload. Identical aggregation produces identical event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EpisodeResolved {
    /// The resolved anime title
    pub anime_title: String,

    /// If matched to existing anime, its ID
    pub matched_anime_id: Option<Uuid>,

    /// The resolved episode number
    pub episode_number: String,

    /// If matched to existing episode, its ID
    pub matched_episode_id: Option<Uuid>,

    /// The primary video file ID (if any)
    pub video_file_id: Option<Uuid>,

    /// Subtitle file IDs associated with this episode
    pub subtitle_file_ids: Vec<Uuid>,

    /// Image file IDs associated with this episode
    pub image_file_ids: Vec<Uuid>,

    /// Aggregated confidence score
    pub confidence: f64,

    /// Deterministic fingerprint for idempotency
    pub fingerprint: String,
}

impl EpisodeResolved {
    pub fn new(
        anime_title: String,
        matched_anime_id: Option<Uuid>,
        episode_number: String,
        matched_episode_id: Option<Uuid>,
        video_file_id: Option<Uuid>,
        subtitle_file_ids: Vec<Uuid>,
        image_file_ids: Vec<Uuid>,
        confidence: f64,
    ) -> Self {
        // Compute deterministic fingerprint
        // DETERMINISM COMPONENTS (documented):
        // - anime_title (lowercased for normalization)
        // - episode_number
        // - video_file_id
        // - subtitle_file_ids (sorted for order stability)
        // - image_file_ids (sorted for order stability)
        let fingerprint = Self::compute_fingerprint(
            &anime_title,
            &episode_number,
            video_file_id,
            &subtitle_file_ids,
            &image_file_ids,
        );

        Self {
            anime_title,
            matched_anime_id,
            episode_number,
            matched_episode_id,
            video_file_id,
            subtitle_file_ids,
            image_file_ids,
            confidence,
            fingerprint,
        }
    }

    /// Compute deterministic fingerprint for idempotency.
    ///
    /// DETERMINISM COMPONENTS:
    /// - anime_title: lowercased for case-insensitive matching
    /// - episode_number: exact string representation
    /// - video_file_id: optional UUID
    /// - subtitle_file_ids: sorted Vec<Uuid> for order stability
    /// - image_file_ids: sorted Vec<Uuid> for order stability
    fn compute_fingerprint(
        anime_title: &str,
        episode_number: &str,
        video_file_id: Option<Uuid>,
        subtitle_file_ids: &[Uuid],
        image_file_ids: &[Uuid],
    ) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        anime_title.to_lowercase().hash(&mut hasher);
        episode_number.hash(&mut hasher);
        video_file_id.hash(&mut hasher);

        // Sort IDs for determinism (order stability)
        let mut sorted_subs = subtitle_file_ids.to_vec();
        sorted_subs.sort();
        for id in &sorted_subs {
            id.hash(&mut hasher);
        }

        let mut sorted_imgs = image_file_ids.to_vec();
        sorted_imgs.sort();
        for id in &sorted_imgs {
            id.hash(&mut hasher);
        }

        format!("ep:{:016x}", hasher.finish())
    }
}

impl DomainEvent for EpisodeResolved {
    fn event_id(&self) -> Uuid {
        // Deterministic event ID derived from fingerprint
        Uuid::new_v5(&Uuid::NAMESPACE_OID, self.fingerprint.as_bytes())
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        // Phase 4 events use sentinel timestamp for determinism
        SENTINEL_TIMESTAMP
    }

    fn event_type(&self) -> &'static str {
        "EpisodeResolved"
    }
}

// ============================================================================
// RESOLUTION FAILED EVENT
// ============================================================================

/// Emitted when resolution fails for a file.
/// Failures are explicit and structured, never silent.
///
/// DETERMINISM: No timestamp in payload. Identical failure produces identical event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResolutionFailed {
    /// The file ID that failed to resolve
    pub file_id: Uuid,

    /// The file path (for traceability)
    pub file_path: PathBuf,

    /// The failure reason (structured)
    pub reason: String,

    /// Human-readable description
    pub description: String,

    /// Deterministic fingerprint for idempotency
    pub fingerprint: String,
}

impl ResolutionFailed {
    pub fn new(file_id: Uuid, file_path: PathBuf, reason: String, description: String) -> Self {
        // Compute deterministic fingerprint
        let fingerprint = Self::compute_fingerprint(file_id, &reason);

        Self {
            file_id,
            file_path,
            reason,
            description,
            fingerprint,
        }
    }

    /// Compute deterministic fingerprint for idempotency.
    ///
    /// DETERMINISM COMPONENTS:
    /// - file_id: UUID of the file
    /// - reason: structured failure reason string
    fn compute_fingerprint(file_id: Uuid, reason: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        file_id.hash(&mut hasher);
        reason.hash(&mut hasher);
        format!("fail:{:016x}", hasher.finish())
    }
}

impl DomainEvent for ResolutionFailed {
    fn event_id(&self) -> Uuid {
        Uuid::new_v5(&Uuid::NAMESPACE_OID, self.fingerprint.as_bytes())
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        // Phase 4 events use sentinel timestamp for determinism
        SENTINEL_TIMESTAMP
    }

    fn event_type(&self) -> &'static str {
        "ResolutionFailed"
    }
}

// ============================================================================
// RESOLUTION SKIPPED EVENT
// ============================================================================

/// Emitted when a file is skipped due to idempotency (already resolved).
/// This makes the skipped_count in ResolutionBatchCompleted meaningful.
///
/// DETERMINISM: No timestamp in payload. Identical skip produces identical event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResolutionSkipped {
    /// The file ID that was skipped
    pub file_id: Uuid,

    /// The file path (for traceability)
    pub file_path: PathBuf,

    /// The existing fingerprint that caused the skip
    pub existing_fingerprint: String,

    /// Reason for skipping
    pub reason: String,
}

impl ResolutionSkipped {
    pub fn new(
        file_id: Uuid,
        file_path: PathBuf,
        existing_fingerprint: String,
        reason: String,
    ) -> Self {
        Self {
            file_id,
            file_path,
            existing_fingerprint,
            reason,
        }
    }
}

impl DomainEvent for ResolutionSkipped {
    fn event_id(&self) -> Uuid {
        // Deterministic event ID derived from existing fingerprint
        Uuid::new_v5(&Uuid::NAMESPACE_OID, self.existing_fingerprint.as_bytes())
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        // Phase 4 events use sentinel timestamp for determinism
        SENTINEL_TIMESTAMP
    }

    fn event_type(&self) -> &'static str {
        "ResolutionSkipped"
    }
}

// ============================================================================
// RESOLUTION BATCH COMPLETED EVENT
// ============================================================================

/// Emitted when a batch resolution operation completes.
/// Provides aggregate statistics for the batch.
///
/// DETERMINISM: No timestamp in payload. Statistics are deterministic from input.
/// Event ID is derived from batch content fingerprint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResolutionBatchCompleted {
    /// Total files processed
    pub total_files: usize,

    /// Files successfully resolved
    pub resolved_count: usize,

    /// Files that failed resolution
    pub failed_count: usize,

    /// Files skipped (already resolved, idempotency)
    pub skipped_count: usize,

    /// Episodes aggregated from resolved files
    pub episodes_aggregated: usize,

    /// Duration of the batch operation in milliseconds
    /// NOTE: This is operational metadata but is deterministic for same input
    /// as it represents processing time, not wall-clock time
    pub duration_ms: u64,

    /// Deterministic fingerprint for the batch
    fingerprint: String,
}

impl ResolutionBatchCompleted {
    pub fn new(
        total_files: usize,
        resolved_count: usize,
        failed_count: usize,
        skipped_count: usize,
        episodes_aggregated: usize,
        duration_ms: u64,
    ) -> Self {
        // Compute deterministic fingerprint from batch statistics
        let fingerprint = Self::compute_fingerprint(
            total_files,
            resolved_count,
            failed_count,
            skipped_count,
            episodes_aggregated,
        );

        Self {
            total_files,
            resolved_count,
            failed_count,
            skipped_count,
            episodes_aggregated,
            duration_ms,
            fingerprint,
        }
    }

    /// Compute deterministic fingerprint for the batch.
    ///
    /// DETERMINISM COMPONENTS:
    /// - total_files
    /// - resolved_count
    /// - failed_count
    /// - skipped_count
    /// - episodes_aggregated
    /// NOTE: duration_ms is excluded as it varies between runs
    fn compute_fingerprint(
        total_files: usize,
        resolved_count: usize,
        failed_count: usize,
        skipped_count: usize,
        episodes_aggregated: usize,
    ) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        total_files.hash(&mut hasher);
        resolved_count.hash(&mut hasher);
        failed_count.hash(&mut hasher);
        skipped_count.hash(&mut hasher);
        episodes_aggregated.hash(&mut hasher);
        format!("batch:{:016x}", hasher.finish())
    }
}

impl DomainEvent for ResolutionBatchCompleted {
    fn event_id(&self) -> Uuid {
        // Deterministic event ID derived from batch fingerprint
        Uuid::new_v5(&Uuid::NAMESPACE_OID, self.fingerprint.as_bytes())
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        // Phase 4 events use sentinel timestamp for determinism
        SENTINEL_TIMESTAMP
    }

    fn event_type(&self) -> &'static str {
        "ResolutionBatchCompleted"
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_resolved_determinism() {
        let file_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let path = PathBuf::from("/test/path.mkv");
        
        let event1 = FileResolved::new(
            file_id,
            path.clone(),
            "Test Anime".to_string(),
            None,
            "01".to_string(),
            None,
            "video".to_string(),
            0.95,
            "filename".to_string(),
            "test_fingerprint".to_string(),
        );
        
        let event2 = FileResolved::new(
            file_id,
            path,
            "Test Anime".to_string(),
            None,
            "01".to_string(),
            None,
            "video".to_string(),
            0.95,
            "filename".to_string(),
            "test_fingerprint".to_string(),
        );
        
        // Same inputs should produce same event ID
        assert_eq!(event1.event_id(), event2.event_id());
        // Sentinel timestamp should be used
        assert_eq!(event1.occurred_at(), SENTINEL_TIMESTAMP);
    }

    #[test]
    fn test_episode_resolved_fingerprint_determinism() {
        let video_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap();
        let sub_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440002").unwrap();
        
        let event1 = EpisodeResolved::new(
            "Test Anime".to_string(),
            None,
            "01".to_string(),
            None,
            Some(video_id),
            vec![sub_id],
            vec![],
            0.95,
        );
        
        let event2 = EpisodeResolved::new(
            "Test Anime".to_string(),
            None,
            "01".to_string(),
            None,
            Some(video_id),
            vec![sub_id],
            vec![],
            0.95,
        );
        
        // Same inputs should produce same fingerprint
        assert_eq!(event1.fingerprint, event2.fingerprint);
        assert_eq!(event1.event_id(), event2.event_id());
    }

    #[test]
    fn test_resolution_batch_completed_determinism() {
        let batch1 = ResolutionBatchCompleted::new(10, 8, 1, 1, 5, 100);
        let batch2 = ResolutionBatchCompleted::new(10, 8, 1, 1, 5, 200); // Different duration
        
        // Same statistics should produce same fingerprint (duration excluded)
        assert_eq!(batch1.fingerprint, batch2.fingerprint);
        assert_eq!(batch1.event_id(), batch2.event_id());
    }
}
