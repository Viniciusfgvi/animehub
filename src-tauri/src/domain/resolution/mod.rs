// src-tauri/src/domain/resolution/mod.rs
//
// Resolution Domain - Phase 4
//
// This module contains value objects representing the outcome of file resolution.
// Resolution transforms detected files into domain intent without committing state.
//
// CRITICAL RULES:
// - All types are pure value objects (immutable)
// - No side effects
// - No persistence
// - No event emission (that's the service's job)
// - Deterministic: same input → same output

pub mod value_objects;

pub use value_objects::{
    ResolutionResult,
    ResolvedFile,
    FileRole,
    ResolutionFailure,
    ResolutionFailureReason,
    ResolvedAnimeIntent,
    ResolvedEpisodeIntent,
    ResolvedEpisodeNumber,
    ResolutionSource,
    ResolutionConfidence,
};
