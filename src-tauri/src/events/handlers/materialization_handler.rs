// src-tauri/src/events/handlers/materialization_handler.rs
//
// Materialization Event Handler - Phase 5
//
// Event handler that listens to resolution events and triggers materialization.
// This is the bridge between the event bus and the MaterializationService.
//
// CRITICAL RULES:
// - Only consumes FileResolved and EpisodeResolved events
// - Delegates all logic to MaterializationService
// - Does not contain business logic
// - Handles errors gracefully without crashing the event bus
// - Uses closure-based subscription (EventHandler is internal to bus)

use std::sync::Arc;

use crate::events::resolution_events::{EpisodeResolved, FileResolved};
use crate::events::EventBus;
use crate::services::materialization_types::MaterializationOutcome;
use crate::services::MaterializationService;

// ============================================================================
// HANDLER REGISTRATION
// ============================================================================

/// Registers all materialization handlers with the event bus.
/// Uses closure-based subscription pattern (EventHandler type is internal).
pub fn register_materialization_handlers(bus: &EventBus, service: Arc<MaterializationService>) {
    // Register FileResolved handler
    let file_service = Arc::clone(&service);
    bus.subscribe::<FileResolved, _>(move |event| {
        handle_file_resolved(&file_service, event);
    });

    // Register EpisodeResolved handler
    let episode_service = Arc::clone(&service);
    bus.subscribe::<EpisodeResolved, _>(move |event| {
        handle_episode_resolved(&episode_service, event);
    });

    println!("[MATERIALIZATION] Handlers registered");
}

// ============================================================================
// FILE RESOLVED HANDLER
// ============================================================================

/// Handles FileResolved events by triggering materialization.
fn handle_file_resolved(service: &MaterializationService, event: &FileResolved) {
    println!(
        "[MATERIALIZATION] Processing FileResolved: file_id={}, anime='{}', episode='{}'",
        event.file_id, event.anime_title, event.episode_number
    );

    match service.materialize_file_resolved(event) {
        Ok(result) => match &result.outcome {
            MaterializationOutcome::Skipped => {
                println!(
                    "[MATERIALIZATION] Skipped (idempotent): file_id={}",
                    event.file_id
                );
            }
            MaterializationOutcome::AnimeCreated => {
                println!(
                    "[MATERIALIZATION] Created anime: id={:?}, title='{}'",
                    result.anime_id, event.anime_title
                );
            }
            MaterializationOutcome::EpisodeCreated => {
                println!(
                    "[MATERIALIZATION] Created episode: id={:?}, number='{}'",
                    result.episode_id, event.episode_number
                );
            }
            MaterializationOutcome::FileLinked => {
                println!(
                    "[MATERIALIZATION] Linked file: file_id={}, episode_id={:?}",
                    event.file_id, result.episode_id
                );
            }
            MaterializationOutcome::AnimeMatched | MaterializationOutcome::EpisodeMatched => {
                println!(
                    "[MATERIALIZATION] Matched existing: anime_id={:?}, episode_id={:?}",
                    result.anime_id, result.episode_id
                );
            }
            MaterializationOutcome::Failed { reason } => {
                eprintln!(
                    "[MATERIALIZATION] Failed: file_id={}, reason={}",
                    event.file_id, reason
                );
            }
        },
        Err(e) => {
            eprintln!(
                "[MATERIALIZATION] Error for file_id={}: {}",
                event.file_id, e
            );
        }
    }
}

// ============================================================================
// EPISODE RESOLVED HANDLER
// ============================================================================

/// Handles EpisodeResolved events by triggering materialization.
fn handle_episode_resolved(service: &MaterializationService, event: &EpisodeResolved) {
    println!(
        "[MATERIALIZATION] Processing EpisodeResolved: anime='{}', episode='{}'",
        event.anime_title, event.episode_number
    );

    match service.materialize_episode_resolved(event) {
        Ok(result) => match &result.outcome {
            MaterializationOutcome::Skipped => {
                println!(
                    "[MATERIALIZATION] Skipped (idempotent): anime='{}', episode='{}'",
                    event.anime_title, event.episode_number
                );
            }
            MaterializationOutcome::AnimeCreated => {
                println!(
                    "[MATERIALIZATION] Created anime from episode: id={:?}",
                    result.anime_id
                );
            }
            MaterializationOutcome::EpisodeCreated => {
                println!(
                    "[MATERIALIZATION] Created episode: id={:?}",
                    result.episode_id
                );
            }
            MaterializationOutcome::FileLinked => {
                println!(
                    "[MATERIALIZATION] Linked files to episode: episode_id={:?}",
                    result.episode_id
                );
            }
            _ => {
                println!("[MATERIALIZATION] Completed: outcome={}", result.outcome);
            }
        },
        Err(e) => {
            eprintln!(
                "[MATERIALIZATION] Error for episode '{}': {}",
                event.episode_number, e
            );
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Integration tests are in materialization_service_tests.rs
}
