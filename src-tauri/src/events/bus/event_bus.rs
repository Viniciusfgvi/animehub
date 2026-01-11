// events/bus/event_bus.rs
//
// Core event bus implementation.
//
// DESIGN PRINCIPLES:
// 1. Synchronous - handlers execute immediately in subscription order
// 2. Deterministic - same events → same result
// 3. Observable - every emission is logged
// 4. Type-safe - events are strongly typed
// 5. No magic - explicit, straightforward code

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::events::types::DomainEvent;

/// Type-erased event handler function
/// Takes a reference to Any (downcasted to concrete event type inside)
type EventHandler = Box<dyn Fn(&dyn Any) + Send + Sync>;

/// The Event Bus
///
/// This is the central coordination point for all domain events.
/// It allows services to emit events and subscribe to events without
/// direct dependencies on each other.
///
/// Key characteristics:
/// - Synchronous execution (no async, no threads)
/// - Handlers execute in subscription order
/// - Type-safe through generics
/// - Observable through logging
pub struct EventBus {
    /// Map from event TypeId to list of handlers
    handlers: Arc<RwLock<HashMap<TypeId, Vec<EventHandler>>>>,

    /// Event emission log (for debugging)
    event_log: Arc<RwLock<Vec<EventLogEntry>>>,
}

/// A logged event for debugging and tracing
#[derive(Debug, Clone)]
pub struct EventLogEntry {
    pub event_type: String,
    pub event_id: String,
    pub occurred_at: String,
    pub handler_count: usize,
}

impl EventBus {
    /// Create a new event bus
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
            event_log: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Subscribe to a specific event type
    ///
    /// Generic parameter E must implement DomainEvent + 'static
    /// The handler function receives a reference to the concrete event
    ///
    /// Handlers are executed in the order they are subscribed.
    ///
    /// Example:
    /// ```ignore
    /// bus.subscribe::<AnimeCreated>(|event| {
    ///     println!("Anime created: {}", event.titulo_principal);
    /// });
    /// ```
    pub fn subscribe<E, F>(&self, handler: F)
    where
        E: DomainEvent + 'static,
        F: Fn(&E) + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<E>();

        // Wrap the typed handler in a type-erased closure
        let wrapped: EventHandler = Box::new(move |event_any: &dyn Any| {
            // Downcast to concrete type
            if let Some(event) = event_any.downcast_ref::<E>() {
                handler(event);
            } else {
                eprintln!(
                    "ERROR: Failed to downcast event in handler for {}",
                    std::any::type_name::<E>()
                );
            }
        });

        let mut handlers = self.handlers.write().unwrap();
        handlers
            .entry(type_id)
            .or_insert_with(Vec::new)
            .push(wrapped);
    }

    /// Emit an event
    ///
    /// This will:
    /// 1. Log the event
    /// 2. Execute all handlers for this event type (in subscription order)
    /// 3. Return immediately (synchronous)
    ///
    /// If a handler panics, the panic is caught and logged, but other handlers
    /// still execute.
    pub fn emit<E>(&self, event: E)
    where
        E: DomainEvent + 'static,
    {
        let type_id = TypeId::of::<E>();

        // Log the event
        let log_entry = EventLogEntry {
            event_type: event.event_type().to_string(),
            event_id: event.event_id().to_string(),
            occurred_at: event.occurred_at().to_rfc3339(),
            handler_count: 0, // will be updated below
        };

        // Get handlers
        let handlers = self.handlers.read().unwrap();
        let event_handlers = handlers.get(&type_id);

        let handler_count = event_handlers.map(|h| h.len()).unwrap_or(0);

        // Update log entry with handler count
        let log_entry = EventLogEntry {
            handler_count,
            ..log_entry
        };

        // Add to event log
        {
            let mut log = self.event_log.write().unwrap();
            log.push(log_entry.clone());
        }

        // Log to console (observable behavior)
        println!(
            "[EVENT] {} (id: {}) | {} handlers",
            log_entry.event_type, log_entry.event_id, log_entry.handler_count
        );

        // Execute handlers
        if let Some(handlers) = event_handlers {
            for (idx, handler) in handlers.iter().enumerate() {
                // Catch panics to prevent one handler from breaking others
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    handler(&event as &dyn Any);
                }));

                if let Err(e) = result {
                    eprintln!(
                        "ERROR: Handler {} for {} panicked: {:?}",
                        idx,
                        event.event_type(),
                        e
                    );
                }
            }
        }
    }

    /// Get the event log (for debugging)
    pub fn get_event_log(&self) -> Vec<EventLogEntry> {
        self.event_log.read().unwrap().clone()
    }

    /// Clear the event log
    pub fn clear_event_log(&self) {
        self.event_log.write().unwrap().clear();
    }

    /// Get the number of subscribers for a specific event type
    pub fn subscriber_count<E>(&self) -> usize
    where
        E: 'static,
    {
        let type_id = TypeId::of::<E>();
        let handlers = self.handlers.read().unwrap();
        handlers.get(&type_id).map(|h| h.len()).unwrap_or(0)
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

// Make EventBus cloneable (shared reference)
impl Clone for EventBus {
    fn clone(&self) -> Self {
        Self {
            handlers: Arc::clone(&self.handlers),
            event_log: Arc::clone(&self.event_log),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::types::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use uuid::Uuid;

    #[test]
    fn test_subscribe_and_emit() {
        let bus = EventBus::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        bus.subscribe::<AnimeCreated, _>(move |_event| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        let event = AnimeCreated::new(Uuid::new_v4(), "Steins;Gate".to_string(), "TV".to_string());

        bus.emit(event);

        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_multiple_handlers_execute_in_order() {
        let bus = EventBus::new();
        let sequence = Arc::new(RwLock::new(Vec::new()));

        let seq1 = Arc::clone(&sequence);
        bus.subscribe::<EpisodeCreated, _>(move |_| {
            seq1.write().unwrap().push(1);
        });

        let seq2 = Arc::clone(&sequence);
        bus.subscribe::<EpisodeCreated, _>(move |_| {
            seq2.write().unwrap().push(2);
        });

        let seq3 = Arc::clone(&sequence);
        bus.subscribe::<EpisodeCreated, _>(move |_| {
            seq3.write().unwrap().push(3);
        });

        let event = EpisodeCreated::new(Uuid::new_v4(), Uuid::new_v4(), "1".to_string());

        bus.emit(event);

        let result = sequence.read().unwrap();
        assert_eq!(*result, vec![1, 2, 3]);
    }

    #[test]
    fn test_event_log_records_emissions() {
        let bus = EventBus::new();

        let event1 =
            AnimeCreated::new(Uuid::new_v4(), "Cowboy Bebop".to_string(), "TV".to_string());

        let event2 = EpisodeCreated::new(Uuid::new_v4(), Uuid::new_v4(), "1".to_string());

        bus.emit(event1);
        bus.emit(event2);

        let log = bus.get_event_log();
        assert_eq!(log.len(), 2);
        assert_eq!(log[0].event_type, "AnimeCreated");
        assert_eq!(log[1].event_type, "EpisodeCreated");
    }

    #[test]
    fn test_subscriber_count() {
        let bus = EventBus::new();

        assert_eq!(bus.subscriber_count::<AnimeCreated>(), 0);

        bus.subscribe::<AnimeCreated, _>(|_| {});
        assert_eq!(bus.subscriber_count::<AnimeCreated>(), 1);

        bus.subscribe::<AnimeCreated, _>(|_| {});
        assert_eq!(bus.subscriber_count::<AnimeCreated>(), 2);

        // Different event type
        assert_eq!(bus.subscriber_count::<EpisodeCreated>(), 0);
    }

    #[test]
    fn test_handler_panic_doesnt_break_bus() {
        let bus = EventBus::new();
        let counter = Arc::new(AtomicUsize::new(0));

        // First handler panics
        bus.subscribe::<AnimeCreated, _>(|_| {
            panic!("Intentional panic");
        });

        // Second handler should still execute
        let counter_clone = Arc::clone(&counter);
        bus.subscribe::<AnimeCreated, _>(move |_| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        let event = AnimeCreated::new(Uuid::new_v4(), "Test".to_string(), "TV".to_string());

        bus.emit(event);

        // Second handler executed despite first one panicking
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }
}
