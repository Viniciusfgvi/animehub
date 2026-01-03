-- AnimeHub Database Schema
-- SQLite 3.x
-- 
-- CRITICAL RULES:
-- 1. UUIDs stored as TEXT
-- 2. Timestamps in ISO 8601 UTC format
-- 3. Enums stored as TEXT with explicit values
-- 4. JSON for flexible/array data
-- 5. Foreign keys with explicit CASCADE behavior
-- 6. No implicit behavior - everything is visible

-- ============================================================================
-- SCHEMA VERSIONING
-- ============================================================================

CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL
);

-- ============================================================================
-- PRIMARY ENTITIES
-- ============================================================================

-- Anime: Root entity for Japanese animation works
CREATE TABLE anime (
    id TEXT PRIMARY KEY,
    titulo_principal TEXT NOT NULL CHECK(length(trim(titulo_principal)) > 0),
    titulos_alternativos TEXT NOT NULL, -- JSON array of strings
    tipo TEXT NOT NULL CHECK(tipo IN ('TV', 'Movie', 'OVA', 'Special')),
    status TEXT NOT NULL CHECK(status IN ('em_exibicao', 'finalizado', 'cancelado')),
    total_episodios INTEGER CHECK(total_episodios IS NULL OR total_episodios > 0),
    data_inicio TEXT, -- ISO 8601 or NULL
    data_fim TEXT,    -- ISO 8601 or NULL
    metadados_livres TEXT NOT NULL, -- JSON object
    criado_em TEXT NOT NULL,
    atualizado_em TEXT NOT NULL
);

CREATE INDEX idx_anime_tipo ON anime(tipo);
CREATE INDEX idx_anime_status ON anime(status);
CREATE INDEX idx_anime_atualizado ON anime(atualizado_em);

-- Episodes: Individual viewing units belonging to an Anime
CREATE TABLE episodes (
    id TEXT PRIMARY KEY,
    anime_id TEXT NOT NULL,
    numero_tipo TEXT NOT NULL CHECK(numero_tipo IN ('regular', 'special')),
    numero_valor TEXT NOT NULL, -- For regular: "1", "2", etc. For special: label
    titulo TEXT,
    duracao_esperada INTEGER CHECK(duracao_esperada IS NULL OR duracao_esperada > 0),
    progresso_atual INTEGER NOT NULL CHECK(progresso_atual >= 0),
    estado TEXT NOT NULL CHECK(estado IN ('nao_visto', 'em_progresso', 'concluido')),
    criado_em TEXT NOT NULL,
    atualizado_em TEXT NOT NULL,
    FOREIGN KEY (anime_id) REFERENCES anime(id) ON DELETE CASCADE
);

CREATE INDEX idx_episodes_anime ON episodes(anime_id);
CREATE INDEX idx_episodes_estado ON episodes(estado);
CREATE INDEX idx_episodes_numero ON episodes(anime_id, numero_tipo, numero_valor);

-- Files: Physical files on disk
CREATE TABLE files (
    id TEXT PRIMARY KEY,
    caminho_absoluto TEXT NOT NULL UNIQUE,
    tipo TEXT NOT NULL CHECK(tipo IN ('video', 'legenda', 'imagem', 'outro')),
    tamanho INTEGER NOT NULL CHECK(tamanho >= 0),
    hash TEXT,
    data_modificacao TEXT NOT NULL,
    origem TEXT NOT NULL CHECK(origem IN ('scan', 'importacao', 'manual')),
    criado_em TEXT NOT NULL,
    atualizado_em TEXT NOT NULL
);

CREATE INDEX idx_files_tipo ON files(tipo);
CREATE INDEX idx_files_caminho ON files(caminho_absoluto);

-- Episode-File association (N:M)
CREATE TABLE episode_files (
    episode_id TEXT NOT NULL,
    file_id TEXT NOT NULL,
    is_primary INTEGER NOT NULL CHECK(is_primary IN (0, 1)),
    criado_em TEXT NOT NULL,
    PRIMARY KEY (episode_id, file_id),
    FOREIGN KEY (episode_id) REFERENCES episodes(id) ON DELETE CASCADE,
    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE
);

CREATE INDEX idx_episode_files_episode ON episode_files(episode_id);
CREATE INDEX idx_episode_files_file ON episode_files(file_id);
CREATE INDEX idx_episode_files_primary ON episode_files(episode_id, is_primary);

-- Subtitles: Transformable subtitle data
CREATE TABLE subtitles (
    id TEXT PRIMARY KEY,
    file_id TEXT NOT NULL,
    formato TEXT NOT NULL CHECK(formato IN ('SRT', 'ASS', 'VTT')),
    idioma TEXT NOT NULL CHECK(length(trim(idioma)) > 0),
    versao INTEGER NOT NULL CHECK(versao > 0),
    eh_original INTEGER NOT NULL CHECK(eh_original IN (0, 1)),
    criado_em TEXT NOT NULL,
    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE
);

CREATE INDEX idx_subtitles_file ON subtitles(file_id);
CREATE INDEX idx_subtitles_idioma ON subtitles(idioma);
CREATE INDEX idx_subtitles_original ON subtitles(eh_original);

-- Subtitle transformations: History of subtitle modifications
CREATE TABLE subtitle_transformations (
    id TEXT PRIMARY KEY,
    subtitle_id_origem TEXT NOT NULL,
    tipo TEXT NOT NULL CHECK(tipo IN ('style', 'timing', 'conversao')),
    parametros_aplicados TEXT NOT NULL, -- JSON object
    criado_em TEXT NOT NULL,
    FOREIGN KEY (subtitle_id_origem) REFERENCES subtitles(id) ON DELETE CASCADE
);

CREATE INDEX idx_subtitle_transformations_origem ON subtitle_transformations(subtitle_id_origem);
CREATE INDEX idx_subtitle_transformations_tipo ON subtitle_transformations(tipo);

-- Collections: User-defined organizational groups
CREATE TABLE collections (
    id TEXT PRIMARY KEY,
    nome TEXT NOT NULL CHECK(length(trim(nome)) > 0),
    descricao TEXT,
    criado_em TEXT NOT NULL
);

-- Anime-Collection association (N:M)
CREATE TABLE anime_collections (
    anime_id TEXT NOT NULL,
    collection_id TEXT NOT NULL,
    criado_em TEXT NOT NULL,
    PRIMARY KEY (anime_id, collection_id),
    FOREIGN KEY (anime_id) REFERENCES anime(id) ON DELETE CASCADE,
    FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE CASCADE
);

CREATE INDEX idx_anime_collections_anime ON anime_collections(anime_id);
CREATE INDEX idx_anime_collections_collection ON anime_collections(collection_id);

-- ============================================================================
-- RELATIONSHIPS & AUXILIARY ENTITIES
-- ============================================================================

-- External references: Links to external services (AniList, etc.)
CREATE TABLE external_references (
    id TEXT PRIMARY KEY,
    anime_id TEXT NOT NULL,
    fonte TEXT NOT NULL CHECK(length(trim(fonte)) > 0),
    external_id TEXT NOT NULL CHECK(length(trim(external_id)) > 0),
    criado_em TEXT NOT NULL,
    UNIQUE(anime_id, fonte),
    FOREIGN KEY (anime_id) REFERENCES anime(id) ON DELETE CASCADE
);

CREATE INDEX idx_external_references_anime ON external_references(anime_id);
CREATE INDEX idx_external_references_fonte ON external_references(fonte);
CREATE INDEX idx_external_references_external_id ON external_references(external_id);

-- Anime aliases: Tracks merge history
CREATE TABLE anime_aliases (
    id TEXT PRIMARY KEY,
    anime_principal_id TEXT NOT NULL,
    anime_alias_id TEXT NOT NULL,
    criado_em TEXT NOT NULL,
    UNIQUE(anime_alias_id),
    FOREIGN KEY (anime_principal_id) REFERENCES anime(id) ON DELETE CASCADE,
    FOREIGN KEY (anime_alias_id) REFERENCES anime(id) ON DELETE CASCADE,
    CHECK(anime_principal_id != anime_alias_id)
);

CREATE INDEX idx_anime_aliases_principal ON anime_aliases(anime_principal_id);
CREATE INDEX idx_anime_aliases_alias ON anime_aliases(anime_alias_id);

-- ============================================================================
-- DERIVED DATA (NOT SOURCE OF TRUTH)
-- ============================================================================

-- ⚠️ DERIVED DATA - CAN BE DELETED WITHOUT AFFECTING DOMAINS
-- Statistics snapshots: Cached aggregate data
CREATE TABLE statistics_snapshots (
    id TEXT PRIMARY KEY,
    tipo TEXT NOT NULL, -- "global", "por_anime:{uuid}", "por_periodo:{start}:{end}"
    valor TEXT NOT NULL, -- JSON snapshot
    gerado_em TEXT NOT NULL
);

CREATE INDEX idx_statistics_tipo ON statistics_snapshots(tipo);
CREATE INDEX idx_statistics_gerado ON statistics_snapshots(gerado_em);

-- ============================================================================
-- EVENT STORE (OPTIONAL - APPEND-ONLY AUDIT LOG)
-- ============================================================================

-- Domain events: Immutable event log for debugging/audit
CREATE TABLE domain_events (
    event_id TEXT PRIMARY KEY,
    event_type TEXT NOT NULL,
    payload TEXT NOT NULL, -- JSON
    occurred_at TEXT NOT NULL
);

CREATE INDEX idx_domain_events_type ON domain_events(event_type);
CREATE INDEX idx_domain_events_occurred ON domain_events(occurred_at);

-- ============================================================================
-- CONSTRAINTS & VALIDATION
-- ============================================================================

-- Ensure progress never exceeds duration (when known)
-- This is a database-level safety check, NOT business logic
-- The domain should prevent this from ever being violated
CREATE TRIGGER check_episode_progress_invariant
BEFORE UPDATE ON episodes
FOR EACH ROW
WHEN NEW.duracao_esperada IS NOT NULL AND NEW.progresso_atual > NEW.duracao_esperada
BEGIN
    SELECT RAISE(ABORT, 'INVARIANT VIOLATION: Progress exceeds duration');
END;

-- Ensure anime dates are logical (if both present)
CREATE TRIGGER check_anime_dates_invariant
BEFORE UPDATE ON anime
FOR EACH ROW
WHEN NEW.data_inicio IS NOT NULL AND NEW.data_fim IS NOT NULL AND NEW.data_inicio > NEW.data_fim
BEGIN
    SELECT RAISE(ABORT, 'INVARIANT VIOLATION: Start date after end date');
END;

-- ============================================================================
-- INITIAL DATA
-- ============================================================================

-- Insert initial schema version
INSERT OR IGNORE INTO schema_version (version, applied_at)
VALUES (1, datetime('now'));