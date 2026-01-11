// src-tauri/src/domain/mod.rs
//
// Domain Root - The Single Source of Truth for Domain API
//
// This file MUST declare all domain modules and re-export their public API.
// All other modules import from `crate::domain::*`

// ============================================================================
// MODULE DECLARATIONS
// ============================================================================

pub mod anime;
pub mod anime_alias;
pub mod collection;
pub mod episode;
pub mod external_reference;
pub mod file;
pub mod resolution;
pub mod statistics;
pub mod subtitle;

// ============================================================================
// PUBLIC API RE-EXPORTS
// ============================================================================

// Anime Domain
pub use anime::{validate_anime, Anime, AnimeStatus, AnimeType};

// Episode Domain
pub use episode::{validate_episode, Episode, EpisodeNumber, EpisodeState};

// File Domain
pub use file::{validate_file, File, FileOrigin, FileType};

// Subtitle Domain
pub use subtitle::{
    validate_subtitle, Subtitle, SubtitleFormat, SubtitleTransformation, TransformationType,
};

// Collection Domain
pub use collection::{validate_collection, Collection};

// Statistics Domain (Derived Data)
pub use statistics::{AnimeStatistics, GlobalStatistics, StatisticsSnapshot, StatisticsType};

// External Reference
pub use external_reference::{validate_external_reference, ExternalReference};

// Anime Alias
pub use anime_alias::{validate_anime_alias, AnimeAlias};

// ============================================================================
// DOMAIN ERROR TYPES
// ============================================================================

use thiserror::Error;

/// Domain-level errors
/// These represent violations of business rules and invariants
#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Invariant violation: {0}")]
    InvariantViolation(String),

    #[error("Progress {progress}s exceeds duration {duration}s")]
    ProgressExceedsDuration { progress: u64, duration: u64 },

    #[error("Invalid state transition: {0}")]
    InvalidStateTransition(String),

    #[error("Entity not found: {0}")]
    NotFound(String),
}

/// Domain result type
pub type DomainResult<T> = Result<T, DomainError>;
