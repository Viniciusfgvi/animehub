# AnimeHub — Technical Baseline & Consolidated State

**Status:** Canonical  
**Last Updated:** 2026-01-10  
**Scope:** Entire AnimeHub codebase  
**Audience:** Engineers (human or AI)

---

## 1. Purpose of This Document

This document defines the **consolidated technical baseline** of the AnimeHub project after:

- Full contract reconciliation
- Removal of hallucinated APIs
- Elimination of invalid tests and mocks
- Structural stabilization of the codebase

Its purpose is to:

- Serve as a **single source of technical truth**
- Prevent architectural or contractual regression
- Define what the system **is**, **is not**, and **must remain**
- Provide a **safe foundation** for continued development

This document is **normative**, not descriptive.

---

## 2. Canonical References

The AnimeHub project is governed by **exactly two canonical artifacts**:

1. **The source code repository**
2. **`Verified AnimeHub Contracts`**

No other documents, comments, or historical intentions override these.

If a conflict arises:
- Contracts > code
- This document constrains interpretation
- Assumptions are invalid

---

## 3. Architectural Closure

The following architectural decisions are **closed and final**:

- Repository-based domain access
- Explicit domain events
- Deterministic materialization
- No implicit orchestration layers
- No generic managers or helpers

Any change to these requires **explicit contract evolution**, not refactoring.

---

## 4. Verified System Guarantees

The system **guarantees** the following invariants:

### 4.1 Determinism
- Fingerprints, IDs, and outcomes are derived exclusively from inputs
- No timestamps, randomness, or external state influence core logic

### 4.2 Idempotency
- Replaying the same event produces no duplicate state
- Materialization is safe to re-run

### 4.3 Explicit Error Propagation
- Errors are never swallowed
- All failures are represented in `AppError` or domain errors
- No silent fallback logic exists

### 4.4 Contract Integrity
- Every method call corresponds to a declared trait method
- Every enum match uses declared variants only
- No implicit fields or dynamic access patterns exist

---

## 5. Explicit Non-Existence (Important)

The following are **explicitly NOT part of the system**:

- `FileRepository::list_by_type`
- `EpisodeRepository::get_linked_files`
- Any mock repository implementations
- Any enum variants not listed in `Verified AnimeHub Contracts`
- Any implicit resolution helpers
- Any background automation or schedulers

Code or tests assuming these are **invalid by definition**.

---

## 6. Test Philosophy

Tests in AnimeHub must:

- Reflect real contracts
- Use real implementations (e.g. SQLite, in-memory)
- Validate invariants, not imagined behavior

Tests that depend on:
- mocks that do not exist
- hypothetical APIs
- future features

are considered **harmful** and must be removed.

---

## 7. Safe Development Guidelines (Going Forward)

Development may safely proceed **only if**:

- New functionality fits within existing contracts  
  **OR**
- Contracts are explicitly evolved in a controlled manner

### 7.1 What Is Allowed
- Adding new services that consume existing repositories
- Adding new commands that respect current traits
- Adding new tests for real behavior
- Improving performance without semantic change

### 7.2 What Is NOT Allowed
- Adding methods to existing traits without a contract revision
- Expanding enums casually
- Introducing helper layers that bypass repositories
- “Fixing” things by weakening invariants

---

## 8. How to Change This Baseline (Strict Process)

Any change to this baseline requires:

1. Explicit identification of the contract change
2. Update to `Verified AnimeHub Contracts`
3. Update to this document
4. Code changes
5. Test updates

Skipping any step invalidates the change.

---

## 9. Final Statement

At the time of writing:

> **AnimeHub is structurally consolidated, contract-consistent, and safe for continued development.**

Any future instability is a result of **explicit deviation**, not hidden technical debt.

---

## 10. Engineering Principle

> A smaller, explicit, truthful system  
> is superior to a larger, ambiguous one.

This document exists to enforce that principle.
