# AnimeHub Event System - Design Document

## Overview

The Event System is the coordination backbone of AnimeHub. It enables services to communicate without direct dependencies, making the system:
- **Decoupled**: Services don't know about each other
- **Observable**: Every action produces traceable events
- **Deterministic**: Same inputs → same outputs
- **Testable**: Services can be tested in isolation

## Architecture Principles

### 1. Events Are Facts, Not Commands

❌ **Wrong** (Command): `CreateAnime { title: "..." }`  
✅ **Right** (Fact): `AnimeCreated { anime_id: ..., title: "..." }`

Events represent something that **has already happened**. They are immutable historical records.

### 2. Synchronous by Design

The event bus executes handlers **immediately** in **subscription order**. This makes the system:
- Predictable (no race conditions)
- Debuggable (step through execution)
- Simple (no async complexity)

When to consider async (future):
- External API calls
- Heavy computation
- I/O-bound operations

But the **core event flow remains synchronous**.

### 3. Type Safety Over Flexibility

We use **concrete types** for every event:
```rust
pub struct AnimeCreated {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub anime_id: Uuid,
    pub titulo_principal: String,
    pub tipo: String,
}
```

No dynamic dispatch. No string-typed events. No `HashMap<String, Any>`.

### 4. One Handler, One Responsibility

Each handler does **exactly one thing**:
- ✅ Update progress
- ✅ Emit derived event
- ✅ Log action
- ❌ Update progress AND send email AND rebuild statistics

If you need multiple things, emit multiple events.

### 5. Services Never Call Services

```rust
// ❌ FORBIDDEN
impl AnimeService {
    fn create_anime(&self) {
        // ...
        self.episode_service.create_episodes(); // NO!
    }
}

// ✅ CORRECT
impl AnimeService {
    fn create_anime(&self, bus: &EventBus) {
        // ...
        bus.emit(AnimeCreated::new(...));
        // EpisodeService will react to this event
    }
}
```

## Event Flow Example

### Scenario: User creates a new anime

```
┌─────────────┐
│     UI      │
└──────┬──────┘
       │ invoke create_anime
       ▼
┌─────────────┐
│ AnimeService│
└──────┬──────┘
       │ emit AnimeCreated
       ▼
┌─────────────┐
│  Event Bus  │
└─────┬───┬───┘
      │   │
      │   └─────────────────┐
      ▼                     ▼
┌─────────────┐    ┌─────────────┐
│  Statistics │    │  External   │
│   Handler   │    │  Integration│
└──────┬──────┘    └──────┬──────┘
       │                  │
       │ emit             │ emit
       │ StatisticsUpdated│ MetadataRequested
       ▼                  ▼
┌─────────────┐    ┌─────────────┐
│  Event Bus  │    │  Event Bus  │
└─────────────┘    └─────────────┘
```

Each handler:
1. Receives event
2. Performs domain operation
3. Emits resulting events (if needed)

## Event Categories

Events are organized by domain:

### File Scanning
- `DirectoryScanned`: Scan completed
- `FileDetected`: Individual file found

### Anime Domain
- `AnimeCreated`: New anime entity
- `AnimeUpdated`: Metadata changed
- `AnimeMerged`: Duplicates resolved

### Episode Domain
- `EpisodeCreated`: New episode
- `FileLinkedToEpisode`: File association
- `EpisodeBecamePlayable`: Has valid video
- `EpisodeProgressUpdated`: Playback progress
- `EpisodeCompleted`: Finished watching

### Playback
- `PlaybackStarted`: Player launched
- `PlaybackProgressUpdated`: Progress tick
- `PlaybackStopped`: Player closed

### Subtitle
- `SubtitleDetected`: Subtitle file found
- `SubtitleStyleApplied`: Style transformation
- `SubtitleTimingAdjusted`: Timing transformation
- `SubtitleVersionCreated`: New derived version

### Statistics
- `StatisticsRebuilt`: Full recalculation
- `StatisticsUpdated`: Incremental update

### External Integration
- `ExternalMetadataRequested`: User wants external data
- `ExternalMetadataFetched`: Data retrieved
- `ExternalMetadataLinked`: Associated with anime

## Usage Patterns

### Pattern 1: Simple Handler

```rust
use animehub::events::*;

#[derive(Clone)]
struct LoggingHandler;

impl EventHandler<AnimeCreated> for LoggingHandler {
    fn handle(&self, event: &AnimeCreated, _bus: &EventBus) {
        println!("New anime: {}", event.titulo_principal);
    }
}

// Register
let bus = EventBus::new();
let handler = LoggingHandler;
handler.register(&bus);
```

### Pattern 2: Handler with State

```rust
#[derive(Clone)]
struct StatisticsHandler {
    repository: Arc<dyn StatisticsRepository>,
}

impl EventHandler<AnimeCreated> for StatisticsHandler {
    fn handle(&self, event: &AnimeCreated, bus: &EventBus) {
        // Update statistics
        self.repository.increment_anime_count();
        
        // Emit result
        bus.emit(StatisticsUpdated::new());
    }
}
```

### Pattern 3: Coordinator (Complex Flow)

```rust
#[derive(Clone)]
struct AnimeCreationCoordinator {
    episode_service: Arc<EpisodeService>,
    external_service: Arc<ExternalIntegrationService>,
}

impl EventHandler<AnimeCreated> for AnimeCreationCoordinator {
    fn handle(&self, event: &AnimeCreated, bus: &EventBus) {
        // Emit sub-tasks as events
        
        if should_create_episodes(&event) {
            bus.emit(CreateDefaultEpisodesRequested {
                anime_id: event.anime_id,
            });
        }
        
        if should_fetch_metadata(&event) {
            bus.emit(ExternalMetadataRequested {
                anime_id: event.anime_id,
                provider: "AniList".to_string(),
            });
        }
    }
}
```

## Testing Strategy

### Unit Test: Single Handler

```rust
#[test]
fn test_statistics_handler() {
    let bus = EventBus::new();
    let repo = Arc::new(MockStatisticsRepository::new());
    let handler = StatisticsHandler { repository: repo.clone() };
    
    handler.register(&bus);
    
    let event = AnimeCreated::new(
        Uuid::new_v4(),
        "Test".to_string(),
        "TV".to_string(),
    );
    
    bus.emit(event);
    
    assert_eq!(repo.anime_count(), 1);
}
```

### Integration Test: Event Flow

```rust
#[test]
fn test_anime_creation_flow() {
    let bus = EventBus::new();
    
    // Register all handlers
    register_all_handlers(&bus);
    
    // Trigger initial event
    bus.emit(AnimeCreated::new(...));
    
    // Verify resulting state
    let log = bus.get_event_log();
    assert!(log.iter().any(|e| e.event_type == "StatisticsUpdated"));
    assert!(log.iter().any(|e| e.event_type == "ExternalMetadataRequested"));
}
```

## Observability

The EventBus provides built-in observability:

### Event Log

```rust
let log = bus.get_event_log();
for entry in log {
    println!("{}: {} | {} handlers",
        entry.occurred_at,
        entry.event_type,
        entry.handler_count
    );
}
```

### Console Output

Every event emission is logged:
```
[EVENT] AnimeCreated (id: 123e4567-...) | 3 handlers
[EVENT] StatisticsUpdated (id: 234f5678-...) | 1 handlers
```

### Tracing (Future)

Event causality can be tracked by storing parent event IDs.

## Anti-Patterns to Avoid

### ❌ God Event

```rust
// BAD: One event doing everything
struct SystemStateChanged {
    anime_updated: bool,
    episodes_updated: bool,
    statistics_updated: bool,
    // ... kitchen sink
}
```

### ❌ Command Masquerading as Event

```rust
// BAD: This is a command, not an event
struct ShouldCreateAnime {
    title: String,
}

// GOOD: Event after the fact
struct AnimeCreated {
    anime_id: Uuid,
    titulo_principal: String,
}
```

### ❌ Handler Doing Too Much

```rust
// BAD: Multiple responsibilities
impl EventHandler<AnimeCreated> for MegaHandler {
    fn handle(&self, event: &AnimeCreated, bus: &EventBus) {
        self.update_statistics();
        self.fetch_external_data();
        self.send_notification();
        self.update_cache();
        self.log_analytics();
        // ... etc
    }
}
```

### ❌ Direct Service Calls

```rust
// BAD: Direct dependency
impl AnimeService {
    fn create(&self) {
        // ...
        self.episode_service.initialize(); // NO!
    }
}

// GOOD: Event-driven
impl AnimeService {
    fn create(&self, bus: &EventBus) {
        // ...
        bus.emit(AnimeCreated::new(...));
        // EpisodeService reacts independently
    }
}
```

## Performance Considerations

### Synchronous Execution

- **Latency**: All handlers execute before `emit()` returns
- **Trade-off**: Predictability > raw speed
- **Mitigation**: Keep handlers fast (< 1ms each)

### Memory

- **Event Log**: Grows unbounded
- **Solution**: Periodic cleanup or size limit
- **Production**: Consider rotating logs

### Scalability

- **Single-threaded**: Current design
- **Future**: Can add async event queue for I/O-bound tasks
- **Hybrid**: Sync for critical path, async for non-critical

## Future Enhancements (Out of Scope for Step 3)

1. **Event Persistence**: Store events for replay
2. **Event Sourcing**: Rebuild state from events
3. **Async Handlers**: For I/O-bound operations
4. **Dead Letter Queue**: For failed events
5. **Event Versioning**: Evolve event schemas
6. **Distributed Events**: Multi-process coordination

These will be considered in later steps if needed.

## Conclusion

The Event System is deliberately **simple and explicit**. It prioritizes:
- **Understanding** over cleverness
- **Debuggability** over performance
- **Correctness** over convenience

This foundation enables AnimeHub to grow without becoming a tangled mess of dependencies.

---

**Next Step**: Implement concrete services that use this event system (Step 5).