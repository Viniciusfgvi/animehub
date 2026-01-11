// src-tauri/src/events/handlers/mod.rs
//
// Event Handlers - INTERNAL MODULE
//
// This module contains handler implementations.
// EventHandler type is internal to the bus module and NOT exported.
//
// Handlers use closure-based subscription via EventBus::subscribe.

// ============================================================================
// MATERIALIZATION HANDLERS (Phase 5)
// ============================================================================

pub mod materialization_handler;

// ============================================================================
// PUBLIC EXPORTS
// ============================================================================

// Only export the registration function, not handler structs
// (handlers use closure-based subscription, not trait-based)
pub use materialization_handler::register_materialization_handlers;
