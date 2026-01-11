//! Critical Collection Invariants:
//!
//! 1. Collections do NOT affect anime state
//! 2. Collections do NOT affect progress
//! 3. Collections are purely organizational
//! 4. Anime can belong to multiple collections
//! 5. Deleting a collection does NOT delete anime
//! 6. Collection name cannot be empty

pub mod entity;

pub use entity::Collection;

use crate::domain::{DomainError, DomainResult};

/// Validates Collection invariants
pub fn validate_collection(collection: &Collection) -> DomainResult<()> {
    if collection.nome.trim().is_empty() {
        return Err(DomainError::InvariantViolation(
            "Collection name cannot be empty".to_string(),
        ));
    }
    Ok(())
}
