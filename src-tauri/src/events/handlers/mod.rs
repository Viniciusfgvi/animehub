// src-tauri/src/events/handlers/mod.rs
//
// Event Handlers - INTERNAL MODULE
//
// This module contains handler implementations but does NOT
// export the EventHandler type (that stays internal to the bus)

use crate::events::bus::EventBus;
use crate::events::types::*;

// ============================================================================
// INTERNAL TRAIT (NOT EXPORTED)
// ============================================================================

// This trait is for internal organization only
// It is NOT re-exported from this module
trait RegisterHandler {
    fn register(self, bus: &EventBus);
}

// ============================================================================
// EXAMPLE HANDLERS (INTERNAL)
// ============================================================================

// Example handler implementation
// In production, these would be in services that subscribe to events
pub struct LoggingHandler;

impl LoggingHandler {
    pub fn new() -> Self {
        Self
    }
    
    pub fn handle_anime_created(&self, event: &AnimeCreated) {
        println!("Anime Created: {} (ID: {})", event.titulo_principal, event.anime_id);
    }
    
    pub fn register(&self, bus: &EventBus) {
        let handler = Self;
        bus.subscribe::<AnimeCreated, _>(move |event| {
            handler.handle_anime_created(event);
        });
    }
}

// More handlers will be added here as services are implemented