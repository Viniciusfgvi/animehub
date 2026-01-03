// src-tauri/src/domain/external_reference.rs
//
// External Reference Entity
//
// Links local anime to external services (AniList, etc.)
// These are auxiliary references, never authoritative.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{DomainError, DomainResult};

/// Represents a link to an external service
/// 
/// CRITICAL INVARIANTS:
/// - Never replaces local data
/// - Can be removed without structural impact
/// - Source never becomes authoritative
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalReference {
    /// Internal identifier
    pub id: Uuid,
    
    /// Local anime this references
    pub anime_id: Uuid,
    
    /// External service name (e.g., "AniList")
    pub fonte: String,
    
    /// ID in the external service
    pub external_id: String,
    
    /// When this reference was created
    pub criado_em: DateTime<Utc>,
}

impl ExternalReference {
    /// Create a new external reference
    pub fn new(anime_id: Uuid, fonte: String, external_id: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            anime_id,
            fonte,
            external_id,
            criado_em: Utc::now(),
        }
    }
}

/// Validates ExternalReference invariants
pub fn validate_external_reference(reference: &ExternalReference) -> DomainResult<()> {
    if reference.fonte.trim().is_empty() {
        return Err(DomainError::InvariantViolation(
            "External reference source cannot be empty".to_string()
        ));
    }
    
    if reference.external_id.trim().is_empty() {
        return Err(DomainError::InvariantViolation(
            "External reference ID cannot be empty".to_string()
        ));
    }
    
    Ok(())
}