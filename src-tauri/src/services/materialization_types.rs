// src-tauri/src/services/materialization_types.rs
//
// Materialization Types - Phase 5
//
// Types for tracking materialization state and ensuring idempotency.
// These types bridge resolution events to domain mutations.
//
// CRITICAL RULES:
// - Fingerprints are deterministic (same input → same fingerprint)
// - Fingerprints are used to detect duplicate materializations
// - No business logic in types
// - All types are serializable for persistence

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use uuid::Uuid;

// ============================================================================
// MATERIALIZATION FINGERPRINT
// ============================================================================

/// A deterministic fingerprint for a resolution event.
/// Used to detect if an event has already been materialized.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MaterializationFingerprint {
    /// The hash of the resolution event's key fields
    hash: String,
}

impl MaterializationFingerprint {
    /// Creates a fingerprint from a FileResolved event's key fields.
    /// The fingerprint is based on:
    /// - file_id (immutable identifier)
    /// - anime_title (normalized)
    /// - episode_number (as string)
    /// - file_role
    pub fn from_file_resolved(
        file_id: Uuid,
        anime_title: &str,
        episode_number: &str,
        file_role: &str,
    ) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(file_id.as_bytes());
        hasher.update(anime_title.to_lowercase().as_bytes());
        hasher.update(episode_number.as_bytes());
        hasher.update(file_role.as_bytes());

        let result = hasher.finalize();
        Self {
            hash: format!("{:x}", result),
        }
    }

    /// Creates a fingerprint from an EpisodeResolved event's key fields.
    pub fn from_episode_resolved(
        anime_title: &str,
        episode_number: &str,
        video_file_id: Option<Uuid>,
    ) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(anime_title.to_lowercase().as_bytes());
        hasher.update(episode_number.as_bytes());
        if let Some(vid_id) = video_file_id {
            hasher.update(vid_id.as_bytes());
        }

        let result = hasher.finalize();
        Self {
            hash: format!("{:x}", result),
        }
    }

    /// Constructs a fingerprint from a stored hash string.
    /// Used when loading from database.
    pub fn from_hash(hash: String) -> Self {
        Self { hash }
    }

    /// Returns the hash string
    pub fn hash(&self) -> &str {
        &self.hash
    }
}

// ============================================================================
// MATERIALIZATION RECORD
// ============================================================================

/// A record of a completed materialization.
/// Stored to prevent duplicate processing of the same resolution event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterializationRecord {
    /// Unique identifier for this record
    pub id: Uuid,

    /// The fingerprint of the resolution event
    pub fingerprint: MaterializationFingerprint,

    /// The type of event that was materialized
    pub event_type: MaterializationEventType,

    /// The source event ID (from resolution)
    pub source_event_id: Uuid,

    /// The resulting anime ID (if created or matched)
    pub anime_id: Option<Uuid>,

    /// The resulting episode ID (if created or matched)
    pub episode_id: Option<Uuid>,

    /// The file ID that was linked (if applicable)
    pub file_id: Option<Uuid>,

    /// The outcome of materialization
    pub outcome: MaterializationOutcome,

    /// When this materialization occurred
    pub materialized_at: DateTime<Utc>,
}

impl MaterializationRecord {
    /// Creates a new materialization record
    pub fn new(
        fingerprint: MaterializationFingerprint,
        event_type: MaterializationEventType,
        source_event_id: Uuid,
        anime_id: Option<Uuid>,
        episode_id: Option<Uuid>,
        file_id: Option<Uuid>,
        outcome: MaterializationOutcome,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            fingerprint,
            event_type,
            source_event_id,
            anime_id,
            episode_id,
            file_id,
            outcome,
            materialized_at: Utc::now(),
        }
    }
}

// ============================================================================
// MATERIALIZATION EVENT TYPE
// ============================================================================

/// The type of resolution event being materialized.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterializationEventType {
    /// FileResolved event
    FileResolved,

    /// EpisodeResolved event
    EpisodeResolved,
}

impl std::fmt::Display for MaterializationEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MaterializationEventType::FileResolved => write!(f, "file_resolved"),
            MaterializationEventType::EpisodeResolved => write!(f, "episode_resolved"),
        }
    }
}

// ============================================================================
// MATERIALIZATION OUTCOME
// ============================================================================

/// The outcome of a materialization operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterializationOutcome {
    /// New anime was created
    AnimeCreated,

    /// Matched to existing anime
    AnimeMatched,

    /// New episode was created
    EpisodeCreated,

    /// Matched to existing episode
    EpisodeMatched,

    /// File was linked to episode
    FileLinked,

    /// Skipped because already materialized (idempotent)
    Skipped,

    /// Failed to materialize
    Failed { reason: String },
}

impl std::fmt::Display for MaterializationOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MaterializationOutcome::AnimeCreated => write!(f, "anime_created"),
            MaterializationOutcome::AnimeMatched => write!(f, "anime_matched"),
            MaterializationOutcome::EpisodeCreated => write!(f, "episode_created"),
            MaterializationOutcome::EpisodeMatched => write!(f, "episode_matched"),
            MaterializationOutcome::FileLinked => write!(f, "file_linked"),
            MaterializationOutcome::Skipped => write!(f, "skipped"),
            MaterializationOutcome::Failed { reason } => write!(f, "failed: {}", reason),
        }
    }
}

// ============================================================================
// MATERIALIZATION DECISION
// ============================================================================

/// The decision made during materialization.
/// This is the internal representation of what action to take.
#[derive(Debug, Clone)]
pub enum MaterializationDecision {
    /// Create a new anime with the given title
    CreateAnime {
        title: String,
        alternative_titles: Vec<String>,
    },

    /// Use an existing anime
    UseExistingAnime { anime_id: Uuid },

    /// Create a new episode for the given anime
    CreateEpisode {
        anime_id: Uuid,
        number: EpisodeNumberDecision,
    },

    /// Use an existing episode
    UseExistingEpisode { episode_id: Uuid },

    /// Link a file to an episode
    LinkFile {
        episode_id: Uuid,
        file_id: Uuid,
        is_primary: bool,
    },

    /// Skip this event (already materialized)
    Skip { reason: String },
}

/// Episode number decision for creation
#[derive(Debug, Clone)]
pub enum EpisodeNumberDecision {
    /// Regular numbered episode
    Regular(u32),

    /// Special episode with label
    Special(String),
}

// ============================================================================
// MATERIALIZATION RESULT
// ============================================================================

/// The result of a materialization operation.
#[derive(Debug, Clone)]
pub struct MaterializationResult {
    /// The fingerprint of the processed event
    pub fingerprint: MaterializationFingerprint,

    /// The anime ID (created or matched)
    pub anime_id: Option<Uuid>,

    /// The episode ID (created or matched)
    pub episode_id: Option<Uuid>,

    /// The file ID that was linked
    pub file_id: Option<Uuid>,

    /// The outcome
    pub outcome: MaterializationOutcome,

    /// Whether this was a new creation or a match
    pub is_new_anime: bool,

    /// Whether this was a new episode creation
    pub is_new_episode: bool,
}

impl MaterializationResult {
    /// Creates a result for a skipped materialization
    pub fn skipped(fingerprint: MaterializationFingerprint, reason: &str) -> Self {
        Self {
            fingerprint,
            anime_id: None,
            episode_id: None,
            file_id: None,
            outcome: MaterializationOutcome::Skipped,
            is_new_anime: false,
            is_new_episode: false,
        }
    }

    /// Creates a result for a failed materialization
    pub fn failed(fingerprint: MaterializationFingerprint, reason: String) -> Self {
        Self {
            fingerprint,
            anime_id: None,
            episode_id: None,
            file_id: None,
            outcome: MaterializationOutcome::Failed { reason },
            is_new_anime: false,
            is_new_episode: false,
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fingerprint_determinism() {
        let file_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

        let fp1 =
            MaterializationFingerprint::from_file_resolved(file_id, "Steins;Gate", "1", "video");

        let fp2 =
            MaterializationFingerprint::from_file_resolved(file_id, "Steins;Gate", "1", "video");

        assert_eq!(fp1, fp2);
        assert_eq!(fp1.hash(), fp2.hash());
    }

    #[test]
    fn test_fingerprint_case_insensitive_title() {
        let file_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

        let fp1 =
            MaterializationFingerprint::from_file_resolved(file_id, "Steins;Gate", "1", "video");

        let fp2 =
            MaterializationFingerprint::from_file_resolved(file_id, "steins;gate", "1", "video");

        assert_eq!(fp1, fp2);
    }

    #[test]
    fn test_fingerprint_different_episodes() {
        let file_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

        let fp1 =
            MaterializationFingerprint::from_file_resolved(file_id, "Steins;Gate", "1", "video");

        let fp2 =
            MaterializationFingerprint::from_file_resolved(file_id, "Steins;Gate", "2", "video");

        assert_ne!(fp1, fp2);
    }

    #[test]
    fn test_fingerprint_from_hash() {
        let original =
            MaterializationFingerprint::from_file_resolved(Uuid::new_v4(), "Test", "1", "video");

        let restored = MaterializationFingerprint::from_hash(original.hash().to_string());
        assert_eq!(original, restored);
    }

    #[test]
    fn test_materialization_outcome_display() {
        assert_eq!(
            MaterializationOutcome::AnimeCreated.to_string(),
            "anime_created"
        );
        assert_eq!(MaterializationOutcome::Skipped.to_string(), "skipped");
        assert_eq!(
            MaterializationOutcome::Failed {
                reason: "test".to_string()
            }
            .to_string(),
            "failed: test"
        );
    }
}
