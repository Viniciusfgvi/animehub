// src-tauri/src/events/materialization_events.rs
//
// Materialization Events - Phase 5
//
// Events emitted during materialization to notify other parts of the system.
// These events represent completed domain mutations.
//
// CRITICAL RULES:
// - Events are facts, not commands
// - Events are immutable
// - These events are emitted AFTER successful persistence
// - No business logic in event types
//
// NOTE: This file extends the existing events system.
// The core domain events (AnimeCreated, EpisodeCreated, FileLinkedToEpisode)
// are already defined in events/types.rs and are reused here.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::types::DomainEvent;

// ============================================================================
// MATERIALIZATION COMPLETED EVENT
// ============================================================================

/// Emitted when a batch of resolution events has been materialized.
/// Provides summary statistics for the materialization operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterializationBatchCompleted {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,

    /// Total events processed
    pub total_events: usize,

    /// New anime created
    pub anime_created_count: usize,

    /// New episodes created
    pub episodes_created_count: usize,

    /// Files linked to episodes
    pub files_linked_count: usize,

    /// Events skipped (already materialized)
    pub skipped_count: usize,

    /// Failed materializations
    pub failed_count: usize,

    /// Duration of the batch operation in milliseconds
    pub duration_ms: u64,
}

impl MaterializationBatchCompleted {
    pub fn new(
        total_events: usize,
        anime_created_count: usize,
        episodes_created_count: usize,
        files_linked_count: usize,
        skipped_count: usize,
        failed_count: usize,
        duration_ms: u64,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            total_events,
            anime_created_count,
            episodes_created_count,
            files_linked_count,
            skipped_count,
            failed_count,
            duration_ms,
        }
    }
}

impl DomainEvent for MaterializationBatchCompleted {
    fn event_id(&self) -> Uuid {
        self.event_id
    }
    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }
    fn event_type(&self) -> &'static str {
        "MaterializationBatchCompleted"
    }
}

// ============================================================================
// MATERIALIZATION RECORD CREATED EVENT
// ============================================================================

/// Emitted when a materialization record is created.
/// Used for audit and debugging purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterializationRecordCreated {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,

    /// The materialization record ID
    pub record_id: Uuid,

    /// The fingerprint hash
    pub fingerprint_hash: String,

    /// The source resolution event ID
    pub source_event_id: Uuid,

    /// The resulting anime ID (if any)
    pub anime_id: Option<Uuid>,

    /// The resulting episode ID (if any)
    pub episode_id: Option<Uuid>,

    /// The file ID that was linked (if any)
    pub file_id: Option<Uuid>,

    /// The outcome as a string
    pub outcome: String,
}

impl MaterializationRecordCreated {
    pub fn new(
        record_id: Uuid,
        fingerprint_hash: String,
        source_event_id: Uuid,
        anime_id: Option<Uuid>,
        episode_id: Option<Uuid>,
        file_id: Option<Uuid>,
        outcome: String,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            record_id,
            fingerprint_hash,
            source_event_id,
            anime_id,
            episode_id,
            file_id,
            outcome,
        }
    }
}

impl DomainEvent for MaterializationRecordCreated {
    fn event_id(&self) -> Uuid {
        self.event_id
    }
    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }
    fn event_type(&self) -> &'static str {
        "MaterializationRecordCreated"
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_completed_event() {
        let event = MaterializationBatchCompleted::new(
            100,  // total
            10,   // anime created
            50,   // episodes created
            80,   // files linked
            5,    // skipped
            5,    // failed
            1500, // duration
        );

        assert_eq!(event.event_type(), "MaterializationBatchCompleted");
        assert_eq!(event.total_events, 100);
        assert_eq!(event.anime_created_count, 10);
    }

    #[test]
    fn test_record_created_event() {
        let event = MaterializationRecordCreated::new(
            Uuid::new_v4(),
            "abc123".to_string(),
            Uuid::new_v4(),
            Some(Uuid::new_v4()),
            Some(Uuid::new_v4()),
            Some(Uuid::new_v4()),
            "anime_created".to_string(),
        );

        assert_eq!(event.event_type(), "MaterializationRecordCreated");
        assert!(event.anime_id.is_some());
    }
}
