// src-tauri/src/domain/anime_alias.rs
//
// Anime Alias Entity
//
// Tracks merge history when duplicate animes are resolved
// Preserves historical relationships

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{DomainError, DomainResult};

/// Represents an alias relationship from anime merge
///
/// CRITICAL INVARIANTS:
/// - Never deleted (preserves history)
/// - One anime becomes "principal", other becomes "alias"
/// - Self-referential aliases are forbidden
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimeAlias {
    /// Internal identifier
    pub id: Uuid,

    /// The principal (surviving) anime
    pub anime_principal_id: Uuid,

    /// The alias (merged into principal) anime
    pub anime_alias_id: Uuid,

    /// When this merge occurred
    pub criado_em: DateTime<Utc>,
}

impl AnimeAlias {
    /// Create a new anime alias relationship
    pub fn new(anime_principal_id: Uuid, anime_alias_id: Uuid) -> Result<Self, String> {
        if anime_principal_id == anime_alias_id {
            return Err("Anime cannot be an alias of itself".to_string());
        }

        Ok(Self {
            id: Uuid::new_v4(),
            anime_principal_id,
            anime_alias_id,
            criado_em: Utc::now(),
        })
    }
}

/// Validates AnimeAlias invariants
pub fn validate_anime_alias(alias: &AnimeAlias) -> DomainResult<()> {
    if alias.anime_principal_id == alias.anime_alias_id {
        return Err(DomainError::InvariantViolation(
            "Anime cannot be an alias of itself".to_string(),
        ));
    }

    Ok(())
}
