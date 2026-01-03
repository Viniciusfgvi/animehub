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
pub mod episode;
pub mod file;
pub mod subtitle;
pub mod collection;
pub mod statistics;
pub mod external_reference;
pub mod anime_alias;
pub mod resolution;

// ============================================================================
// PUBLIC API RE-EXPORTS
// ============================================================================

// Anime Domain
pub use anime::{Anime, AnimeType, AnimeStatus, validate_anime};

// Episode Domain
pub use episode::{Episode, EpisodeNumber, EpisodeState, validate_episode};

// File Domain
pub use file::{File, FileType, FileOrigin, validate_file};

// Subtitle Domain
pub use subtitle::{
    Subtitle, 
    SubtitleFormat, 
    SubtitleTransformation, 
    TransformationType,
    validate_subtitle
};

// Collection Domain
pub use collection::{Collection, validate_collection};

// Statistics Domain (Derived Data)
pub use statistics::{StatisticsSnapshot, StatisticsType, GlobalStatistics, AnimeStatistics};

// External Reference
pub use external_reference::{ExternalReference, validate_external_reference};

// Anime Alias
pub use anime_alias::{AnimeAlias, validate_anime_alias};

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