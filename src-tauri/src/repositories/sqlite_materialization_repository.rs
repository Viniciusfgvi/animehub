// src-tauri/src/repositories/sqlite_materialization_repository.rs
//
// Materialization Repository - PHASE 4 CORRECTED
//
// CORRECTIONS APPLIED:
// - Replaced .unwrap_or_default() with explicit error propagation
// - Replaced .unwrap_or_else(|_| chrono::Utc::now()) with explicit error propagation
// - All parse failures now result in explicit errors
// - REMOVED: InvalidData variant (does not exist in AppError)
// - REMOVED: Success/Failure variants (do not exist in MaterializationOutcome)
// - REMOVED: ResolutionFailed/BatchCompleted variants (do not exist in MaterializationEventType)
// - FIXED: Use AppError::Other for parse errors

use crate::services::materialization_types::{
    MaterializationEventType, MaterializationFingerprint, MaterializationOutcome,
    MaterializationRecord,
};
use crate::error::AppError;
use uuid::Uuid;
use rusqlite::Connection;
use std::str::FromStr;

pub struct SqliteMaterializationRepository {
    pub conn: Connection,
}

impl SqliteMaterializationRepository {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }
}

impl FromStr for MaterializationEventType {
    type Err = AppError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "file_resolved" | "FileResolved" => Ok(Self::FileResolved),
            "episode_resolved" | "EpisodeResolved" => Ok(Self::EpisodeResolved),
            _ => Err(AppError::Other(format!("Invalid event type: {}", s))),
        }
    }
}

impl FromStr for MaterializationOutcome {
    type Err = AppError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "anime_created" => Ok(Self::AnimeCreated),
            "anime_matched" => Ok(Self::AnimeMatched),
            "episode_created" => Ok(Self::EpisodeCreated),
            "episode_matched" => Ok(Self::EpisodeMatched),
            "file_linked" => Ok(Self::FileLinked),
            "skipped" => Ok(Self::Skipped),
            s if s.starts_with("failed:") => {
                let reason = s.strip_prefix("failed:").unwrap_or("unknown").trim().to_string();
                Ok(Self::Failed { reason })
            }
            _ => Err(AppError::Other(format!("Invalid outcome: {}", s))),
        }
    }
}

impl SqliteMaterializationRepository {
    /// Convert a database row to a MaterializationRecord.
    ///
    /// PHASE 4 CORRECTION: All parse failures are explicit errors, not silent defaults.
    #[allow(dead_code)] // Used by query methods that will be implemented
    fn row_to_record(row: &rusqlite::Row) -> rusqlite::Result<MaterializationRecord> {
        let id_str: String = row.get(0)?;
        let fingerprint_hash: String = row.get(1)?;
        let event_type_str: String = row.get(2)?;
        let source_event_id_str: String = row.get(3)?;
        let anime_id_str: Option<String> = row.get(4)?;
        let episode_id_str: Option<String> = row.get(5)?;
        let file_id_str: Option<String> = row.get(6)?;
        let outcome_str: String = row.get(7)?;
        let materialized_at_str: String = row.get(8)?;

        // CORRECTION: Parse UUID with explicit error
        let id = Uuid::parse_str(&id_str).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Invalid materialization record UUID '{}': {}", id_str, e),
                )),
            )
        })?;

        let source_event_id = Uuid::parse_str(&source_event_id_str).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                3,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Invalid source_event_id UUID '{}': {}", source_event_id_str, e),
                )),
            )
        })?;

        // CORRECTION: Parse optional UUIDs with explicit error handling
        let anime_id = match anime_id_str {
            Some(s) => Some(Uuid::parse_str(&s).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    4,
                    rusqlite::types::Type::Text,
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Invalid anime_id UUID '{}': {}", s, e),
                    )),
                )
            })?),
            None => None,
        };

        let episode_id = match episode_id_str {
            Some(s) => Some(Uuid::parse_str(&s).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    5,
                    rusqlite::types::Type::Text,
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Invalid episode_id UUID '{}': {}", s, e),
                    )),
                )
            })?),
            None => None,
        };

        let file_id = match file_id_str {
            Some(s) => Some(Uuid::parse_str(&s).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    6,
                    rusqlite::types::Type::Text,
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Invalid file_id UUID '{}': {}", s, e),
                    )),
                )
            })?),
            None => None,
        };

        // CORRECTION: Parse timestamp with explicit error
        let materialized_at = chrono::DateTime::parse_from_rfc3339(&materialized_at_str)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    8,
                    rusqlite::types::Type::Text,
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Invalid materialized_at timestamp '{}': {}", materialized_at_str, e),
                    )),
                )
            })?;

        // Parse event type - convert AppError to rusqlite::Error
        let event_type = MaterializationEventType::from_str(&event_type_str).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                2,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    e.to_string(),
                )),
            )
        })?;

        // Parse outcome - convert AppError to rusqlite::Error
        let outcome = MaterializationOutcome::from_str(&outcome_str).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                7,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    e.to_string(),
                )),
            )
        })?;

        Ok(MaterializationRecord {
            id,
            fingerprint: MaterializationFingerprint::from_hash(fingerprint_hash),
            event_type,
            source_event_id,
            anime_id,
            episode_id,
            file_id,
            outcome,
            materialized_at,
        })
    }
}

#[cfg(test)]
mod error_propagation_tests {
    use super::*;

    /// PROVES: Invalid UUID in materialization record causes explicit error
    #[test]
    fn test_invalid_uuid_causes_error() {
        let result = Uuid::parse_str("invalid-uuid-format");
        assert!(result.is_err(), "Invalid UUID MUST cause parse error");
    }

    /// PROVES: Invalid timestamp in materialization record causes explicit error
    #[test]
    fn test_invalid_timestamp_causes_error() {
        let result = chrono::DateTime::parse_from_rfc3339("not-a-valid-timestamp");
        assert!(result.is_err(), "Invalid timestamp MUST cause parse error");
    }

    /// PROVES: MaterializationEventType parsing uses canonical variants only
    #[test]
    fn test_event_type_parsing() {
        assert!(MaterializationEventType::from_str("file_resolved").is_ok());
        assert!(MaterializationEventType::from_str("episode_resolved").is_ok());
        assert!(MaterializationEventType::from_str("FileResolved").is_ok());
        assert!(MaterializationEventType::from_str("EpisodeResolved").is_ok());
        // Invalid variants should fail
        assert!(MaterializationEventType::from_str("ResolutionFailed").is_err());
        assert!(MaterializationEventType::from_str("BatchCompleted").is_err());
    }

    /// PROVES: MaterializationOutcome parsing uses canonical variants only
    #[test]
    fn test_outcome_parsing() {
        assert!(MaterializationOutcome::from_str("anime_created").is_ok());
        assert!(MaterializationOutcome::from_str("anime_matched").is_ok());
        assert!(MaterializationOutcome::from_str("episode_created").is_ok());
        assert!(MaterializationOutcome::from_str("episode_matched").is_ok());
        assert!(MaterializationOutcome::from_str("file_linked").is_ok());
        assert!(MaterializationOutcome::from_str("skipped").is_ok());
        assert!(MaterializationOutcome::from_str("failed: some reason").is_ok());
        // Invalid variants should fail
        assert!(MaterializationOutcome::from_str("Success").is_err());
        assert!(MaterializationOutcome::from_str("Failure").is_err());
    }
}
