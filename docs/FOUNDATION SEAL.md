Purpose

This document formally defines which parts of the AnimeHub codebase are SEALED as of the completion of the Foundation Phase (Phase 3).

Sealed components are considered stable architectural boundaries. They are not frozen forever, but must not be modified casually. Any change to a sealed component requires an explicit architectural review.

This document exists to:

Prevent accidental architectural erosion

Stop unnecessary refactors driven by tooling or tests

Provide clarity for future contributors (human or AI)

Definition of “SEALED”

A sealed component:

Is architecturally correct and complete for the current phase

Defines contracts relied upon by other layers

Must not be changed to satisfy outdated tests or conveniences

Can only be modified for explicit, justified reasons (bug, design flaw, new phase decision)

Sealing is a process decision, not a technical restriction.

🔒 SEALED COMPONENTS
1. Domain Layer (STRICTLY SEALED)

Location: src-tauri/src/domain/

These files define the core business model. They are pure, dependency-free, and invariant-driven.

Sealed directories and files:

domain/anime/

domain/episode/

domain/file/

domain/subtitle/

domain/collection/

domain/statistics/

domain/external_reference.rs

domain/anime_alias.rs

Rules:

❌ No database access

❌ No filesystem access

❌ No event bus access

❌ No UI or service concerns

✅ Only business rules and invariants

2. Repository Contracts (SEALED)

Location: src-tauri/src/repositories/

These traits define the stable boundary between services and persistence.

Sealed files:

anime_repository.rs

episode_repository.rs

file_repository.rs

subtitle_repository.rs

collection_repository.rs

external_reference_repository.rs

anime_alias_repository.rs

statistics_repository.rs

Rules:

❌ No business logic

❌ No cross-repository calls

❌ No service calls

✅ Traits only

3. Database Schema (SEALED)

Location: src-tauri/schema.sql

This file defines the canonical SQLite schema.

Rules:

❌ No breaking changes

❌ No column removal or type mutation

✅ Only additive migrations allowed in future phases

4. Event System (SEALED)

Location: src-tauri/src/events/

Sealed files:

events/types.rs

events/bus/event_bus.rs

Rules:

❌ Existing events must not change

❌ Event semantics are immutable

✅ New events may be added

5. Service Responsibilities (SEALED)

Location: src-tauri/src/services/

Sealed service files:

anime_service.rs

episode_service.rs

file_service.rs

playback_service.rs

statistics_service.rs

external_integration_service.rs

subtitle_service.rs

Rules:

❌ No business logic inside services

❌ Services must not call other services directly

✅ Services orchestrate domain + repositories + events

⚠️ LEGACY / NON-CANONICAL CODE (DO NOT USE)
Legacy SQLite Implementations

Location: src-tauri/src/repositories/sqlite/

Files:

sqlite_episode_repository.rs

sqlite_file_repository.rs

sqlite_statistics_repository.rs

Status:

⚠️ Legacy

❌ Not used by the current architecture

❌ Must not be referenced by new code

✅ Kept only for historical context

🧪 TEST STATUS CLARIFICATION
Valid Tests (Authoritative)

Domain invariant tests (inside domain modules)

Event bus tests

Database migration tests

Infrastructure utility tests

Legacy / Out-of-Phase Tests

Early repository integration tests

Tests referencing outdated module paths

Tests assuming deprecated helpers

These tests:

❌ Are not required to pass

❌ Must not drive architectural changes

⚠️ Are considered historical

What Future Development MAY Do

✅ Add new services ✅ Add new service methods ✅ Add new events ✅ Add UI layers ✅ Add external integrations (AniList, MPV, etc.) ✅ Add integration tests at application boundaries

What Future Development MUST NOT Do

❌ Modify sealed domain models casually ❌ Change repository contracts without review ❌ Break schema compatibility ❌ Add business logic to repositories ❌ Create service-to-service dependencies

Phase Status

Foundation Phase (Phase 3): CLOSED & SEALED

This document is the authoritative reference for what is considered stable.

All future work builds around this foundation, not inside it.