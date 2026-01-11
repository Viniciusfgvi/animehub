// src-tauri/src/repositories/materialization_repository.rs
//
// Materialization Repository - Phase 5
//
// Repository trait for persisting materialization records.
// Used to track which resolution events have been materialized.
//
// CRITICAL RULES:
// - This is a NEW repository for Phase 5 only
// - Does not modify existing repository contracts
// - Provides idempotency tracking

use uuid::Uuid;

use crate::error::AppResult;
use crate::services::materialization_types::{MaterializationFingerprint, MaterializationRecord};

/// Repository trait for materialization records.
/// Tracks which resolution events have been materialized to ensure idempotency.
pub trait MaterializationRepository: Send + Sync {
    /// Check if a fingerprint has already been materialized
    fn exists_by_fingerprint(&self, fingerprint: &MaterializationFingerprint) -> AppResult<bool>;

    /// Get a materialization record by fingerprint
    fn get_by_fingerprint(
        &self,
        fingerprint: &MaterializationFingerprint,
    ) -> AppResult<Option<MaterializationRecord>>;

    /// Get a materialization record by ID
    fn get_by_id(&self, id: Uuid) -> AppResult<Option<MaterializationRecord>>;

    /// Get a materialization record by source event ID
    fn get_by_source_event_id(&self, event_id: Uuid) -> AppResult<Option<MaterializationRecord>>;

    /// Save a new materialization record
    fn save(&self, record: &MaterializationRecord) -> AppResult<()>;

    /// List all materialization records for an anime
    fn list_by_anime_id(&self, anime_id: Uuid) -> AppResult<Vec<MaterializationRecord>>;

    /// List all materialization records for an episode
    fn list_by_episode_id(&self, episode_id: Uuid) -> AppResult<Vec<MaterializationRecord>>;

    /// Count total materialization records
    fn count(&self) -> AppResult<usize>;
}
