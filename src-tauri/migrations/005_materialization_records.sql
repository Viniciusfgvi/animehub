-- Migration: 005_materialization_records.sql
-- Phase 5: Domain Materialization
--
-- Creates the materialization_records table for idempotency tracking.
-- This table stores fingerprints of processed resolution events to prevent
-- duplicate entity creation on replay.
--
-- CRITICAL: This is an ADDITIVE migration. It does not modify existing tables.

-- ============================================================================
-- MATERIALIZATION RECORDS TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS materialization_records (
    -- Primary key
    id TEXT PRIMARY KEY,
    
    -- Fingerprint hash for idempotency checking
    -- This is a SHA-256 hash of the resolution event's key fields
    fingerprint_hash TEXT NOT NULL UNIQUE,
    
    -- Type of resolution event that was materialized
    -- Values: 'file_resolved', 'episode_resolved'
    event_type TEXT NOT NULL,
    
    -- The source resolution event ID (for traceability)
    source_event_id TEXT NOT NULL,
    
    -- The resulting anime ID (if created or matched)
    anime_id TEXT,
    
    -- The resulting episode ID (if created or matched)
    episode_id TEXT,
    
    -- The file ID that was linked (if applicable)
    file_id TEXT,
    
    -- The outcome of materialization
    -- Values: 'anime_created', 'anime_matched', 'episode_created', 
    --         'episode_matched', 'file_linked', 'skipped', 'failed: <reason>'
    outcome TEXT NOT NULL,
    
    -- When this materialization occurred (ISO 8601 format)
    materialized_at TEXT NOT NULL,
    
    -- Foreign key constraints (soft references, not enforced)
    -- These allow the record to persist even if the referenced entity is deleted
    CONSTRAINT fk_anime FOREIGN KEY (anime_id) REFERENCES anime(id) ON DELETE SET NULL,
    CONSTRAINT fk_episode FOREIGN KEY (episode_id) REFERENCES episodes(id) ON DELETE SET NULL,
    CONSTRAINT fk_file FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE SET NULL
);

-- ============================================================================
-- INDEXES
-- ============================================================================

-- Primary lookup by fingerprint (for idempotency check)
CREATE INDEX IF NOT EXISTS idx_materialization_fingerprint 
    ON materialization_records(fingerprint_hash);

-- Lookup by source event (for debugging and audit)
CREATE INDEX IF NOT EXISTS idx_materialization_source_event 
    ON materialization_records(source_event_id);

-- Lookup by anime (for listing all materializations for an anime)
CREATE INDEX IF NOT EXISTS idx_materialization_anime 
    ON materialization_records(anime_id);

-- Lookup by episode (for listing all materializations for an episode)
CREATE INDEX IF NOT EXISTS idx_materialization_episode 
    ON materialization_records(episode_id);

-- Lookup by file (for tracing file linkage history)
CREATE INDEX IF NOT EXISTS idx_materialization_file 
    ON materialization_records(file_id);

-- Lookup by timestamp (for audit and cleanup)
CREATE INDEX IF NOT EXISTS idx_materialization_timestamp 
    ON materialization_records(materialized_at);

-- ============================================================================
-- COMMENTS
-- ============================================================================

-- This table is designed for:
-- 1. Idempotency: Prevent duplicate entity creation on event replay
-- 2. Traceability: Link every domain entity to its source resolution event
-- 3. Debugging: Understand why an entity was created and from what source
-- 4. Audit: Track all materialization operations over time
--
-- The fingerprint_hash is computed from:
-- - For FileResolved: file_id + anime_title (lowercase) + episode_number + file_role
-- - For EpisodeResolved: anime_title (lowercase) + episode_number + video_file_id
--
-- This ensures that:
-- - The same file resolved to the same anime/episode will not be processed twice
-- - Different files for the same episode will each be processed once
-- - Replaying the entire event stream is safe and idempotent
