// src-tauri/src/services/materialization_service.rs
//
// Materialization Service - Phase 5
//
// Consumes resolution events and materializes domain entities.
// This is the bridge between resolution knowledge and domain reality.
//
// CRITICAL RULES:
// - Consumes ONLY resolution events (FileResolved, EpisodeResolved)
// - Creates Anime/Episode entities via existing repositories
// - Links files to episodes via existing repositories
// - Enforces invariants at write time
// - Ensures idempotency via fingerprinting
// - Emits domain events for created/linked entities
// - Does NOT modify Resolution (Phase 4) code
// - Does NOT bypass repository contracts
//
// PHASE 5 GUARANTEES:
// - Deterministic: same events → same domain state
// - Idempotent: replaying events does not duplicate entities
// - Traceable: every entity creation is linked to a resolution event

use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

use super::materialization_types::{
    EpisodeNumberDecision, MaterializationDecision, MaterializationEventType,
    MaterializationFingerprint, MaterializationOutcome, MaterializationRecord,
    MaterializationResult,
};
use crate::domain::anime::{validate_anime, Anime, AnimeStatus, AnimeType};
use crate::domain::episode::{validate_episode, Episode, EpisodeNumber, EpisodeState};
use crate::domain::file::FileType;
use crate::error::{AppError, AppResult};
use crate::events::resolution_events::{EpisodeResolved, FileResolved};
use crate::events::DomainEvent;
use crate::events::types::{AnimeCreated, EpisodeCreated, FileLinkedToEpisode};
use crate::events::EventBus;
use crate::repositories::anime_repository::AnimeRepository;
use crate::repositories::EpisodeRepository;

use crate::repositories::FileRepository;

use crate::repositories::materialization_repository::MaterializationRepository;

// ============================================================================
// MATERIALIZATION SERVICE
// ============================================================================

pub struct MaterializationService {
    anime_repo: Arc<dyn AnimeRepository>,
    episode_repo: Arc<dyn EpisodeRepository>,
    file_repo: Arc<dyn FileRepository>,
    materialization_repo: Arc<dyn MaterializationRepository>,
    event_bus: Arc<EventBus>,
}

impl MaterializationService {
    pub fn new(
        anime_repo: Arc<dyn AnimeRepository>,
        episode_repo: Arc<dyn EpisodeRepository>,
        file_repo: Arc<dyn FileRepository>,
        materialization_repo: Arc<dyn MaterializationRepository>,
        event_bus: Arc<EventBus>,
    ) -> Self {
        Self {
            anime_repo,
            episode_repo,
            file_repo,
            materialization_repo,
            event_bus,
        }
    }

    // ========================================================================
    // PUBLIC API: EVENT HANDLERS
    // ========================================================================

    /// Materialize a FileResolved event.
    /// This is the primary entry point for Phase 5.
    pub fn materialize_file_resolved(
        &self,
        event: &FileResolved,
    ) -> AppResult<MaterializationResult> {
        // Step 1: Compute fingerprint
        let fingerprint = MaterializationFingerprint::from_file_resolved(
            event.file_id,
            &event.anime_title,
            &event.episode_number,
            &event.file_role,
        );

        // Step 2: Check idempotency
        if self
            .materialization_repo
            .exists_by_fingerprint(&fingerprint)?
        {
            return Ok(MaterializationResult::skipped(
                fingerprint,
                "Already materialized",
            ));
        }

        // Step 3: Determine anime decision
        let anime_decision = self.decide_anime(&event.anime_title, event.matched_anime_id)?;

        // Step 4: Execute anime decision and get anime_id
        let (anime_id, is_new_anime) = self.execute_anime_decision(&anime_decision)?;

        // Step 5: Determine episode decision
        let episode_decision =
            self.decide_episode(anime_id, &event.episode_number, event.matched_episode_id)?;

        // Step 6: Execute episode decision and get episode_id
        let (episode_id, is_new_episode) = self.execute_episode_decision(&episode_decision)?;

        // Step 7: Link file to episode (if video or subtitle)
        let file_linked =
            self.link_file_if_applicable(episode_id, event.file_id, &event.file_role)?;

        // Step 8: Determine final outcome
        let outcome = self.determine_outcome(is_new_anime, is_new_episode, file_linked);

        // Step 9: Record materialization
        let record = MaterializationRecord::new(
            fingerprint.clone(),
            MaterializationEventType::FileResolved,
            event.event_id(),
            Some(anime_id),
            Some(episode_id),
            Some(event.file_id),
            outcome.clone(),
        );
        self.materialization_repo.save(&record)?;

        // Step 10: Return result
        Ok(MaterializationResult {
            fingerprint,
            anime_id: Some(anime_id),
            episode_id: Some(episode_id),
            file_id: Some(event.file_id),
            outcome,
            is_new_anime,
            is_new_episode,
        })
    }

    /// Materialize an EpisodeResolved event.
    /// Handles aggregated episode resolution.
    pub fn materialize_episode_resolved(
        &self,
        event: &EpisodeResolved,
    ) -> AppResult<MaterializationResult> {
        // Step 1: Compute fingerprint
        let fingerprint = MaterializationFingerprint::from_episode_resolved(
            &event.anime_title,
            &event.episode_number,
            event.video_file_id,
        );

        // Step 2: Check idempotency
        if self
            .materialization_repo
            .exists_by_fingerprint(&fingerprint)?
        {
            return Ok(MaterializationResult::skipped(
                fingerprint,
                "Already materialized",
            ));
        }

        // Step 3: Determine anime decision
        let anime_decision = self.decide_anime(&event.anime_title, event.matched_anime_id)?;

        // Step 4: Execute anime decision
        let (anime_id, is_new_anime) = self.execute_anime_decision(&anime_decision)?;

        // Step 5: Determine episode decision
        let episode_decision =
            self.decide_episode(anime_id, &event.episode_number, event.matched_episode_id)?;

        // Step 6: Execute episode decision
        let (episode_id, is_new_episode) = self.execute_episode_decision(&episode_decision)?;

        // Step 7: Link video file if present
        if let Some(video_id) = event.video_file_id {
            self.link_file_if_applicable(episode_id, video_id, "video")?;
        }

        // Step 8: Link subtitle files
        for sub_id in &event.subtitle_file_ids {
            self.link_file_if_applicable(episode_id, *sub_id, "subtitle")?;
        }

        // Step 9: Determine outcome
        let outcome = self.determine_outcome(
            is_new_anime,
            is_new_episode,
            event.video_file_id.is_some() || !event.subtitle_file_ids.is_empty(),
        );

        // Step 10: Record materialization
        let record = MaterializationRecord::new(
            fingerprint.clone(),
            MaterializationEventType::EpisodeResolved,
            event.event_id(),
            Some(anime_id),
            Some(episode_id),
            event.video_file_id,
            outcome.clone(),
        );
        self.materialization_repo.save(&record)?;

        Ok(MaterializationResult {
            fingerprint,
            anime_id: Some(anime_id),
            episode_id: Some(episode_id),
            file_id: event.video_file_id,
            outcome,
            is_new_anime,
            is_new_episode,
        })
    }

    // ========================================================================
    // DECISION LOGIC
    // ========================================================================

    /// Decide what to do about the anime: create new or use existing.
    fn decide_anime(
        &self,
        title: &str,
        matched_id: Option<Uuid>,
    ) -> AppResult<MaterializationDecision> {
        // If resolution already matched an anime, use it
        if let Some(anime_id) = matched_id {
            // Verify the anime still exists
            if self.anime_repo.get_by_id(anime_id)?.is_some() {
                return Ok(MaterializationDecision::UseExistingAnime { anime_id });
            }
        }

        // Try to find by title (case-insensitive)
        if let Some(existing) = self.find_anime_by_title(title)? {
            return Ok(MaterializationDecision::UseExistingAnime {
                anime_id: existing.id,
            });
        }

        // No match found, create new
        Ok(MaterializationDecision::CreateAnime {
            title: title.to_string(),
            alternative_titles: Vec::new(),
        })
    }

    /// Decide what to do about the episode: create new or use existing.
    fn decide_episode(
        &self,
        anime_id: Uuid,
        episode_number: &str,
        matched_id: Option<Uuid>,
    ) -> AppResult<MaterializationDecision> {
        // If resolution already matched an episode, use it
        if let Some(episode_id) = matched_id {
            // Verify the episode still exists
            if self.episode_repo.get_by_id(episode_id)?.is_some() {
                return Ok(MaterializationDecision::UseExistingEpisode { episode_id });
            }
        }

        // Parse episode number
        let number_decision = self.parse_episode_number(episode_number);

        // Try to find existing episode
        if let Some(existing) = self.find_episode_by_number(anime_id, &number_decision)? {
            return Ok(MaterializationDecision::UseExistingEpisode {
                episode_id: existing.id,
            });
        }

        // No match found, create new
        Ok(MaterializationDecision::CreateEpisode {
            anime_id,
            number: number_decision,
        })
    }

    // ========================================================================
    // EXECUTION LOGIC
    // ========================================================================

    /// Execute an anime decision, returning the anime_id and whether it was newly created.
    fn execute_anime_decision(
        &self,
        decision: &MaterializationDecision,
    ) -> AppResult<(Uuid, bool)> {
        match decision {
            MaterializationDecision::UseExistingAnime { anime_id } => Ok((*anime_id, false)),
            MaterializationDecision::CreateAnime {
                title,
                alternative_titles,
            } => {
                let now = Utc::now();
                let anime = Anime {
                    id: Uuid::new_v4(),
                    titulo_principal: title.clone(),
                    titulos_alternativos: alternative_titles.clone(),
                    tipo: AnimeType::TV,
                    status: AnimeStatus::EmExibicao,
                    total_episodios: None,
                    data_inicio: None,
                    data_fim: None,
                    metadados_livres: serde_json::Value::Null,
                    criado_em: now,
                    atualizado_em: now,
                };

                // Validate invariants before persisting
                validate_anime(&anime)?;

                // Persist
                self.anime_repo.save(&anime)?;

                // Emit event
                self.event_bus.emit(AnimeCreated::new(
                    anime.id,
                    anime.titulo_principal.clone(),
                    anime.tipo.to_string(),
                ));

                Ok((anime.id, true))
            }
            _ => Err(AppError::Other("Invalid anime decision".to_string())),
        }
    }

    /// Execute an episode decision, returning the episode_id and whether it was newly created.
    fn execute_episode_decision(
        &self,
        decision: &MaterializationDecision,
    ) -> AppResult<(Uuid, bool)> {
        match decision {
            MaterializationDecision::UseExistingEpisode { episode_id } => Ok((*episode_id, false)),
            MaterializationDecision::CreateEpisode { anime_id, number } => {
                let now = Utc::now();
                let (numero, titulo) = match number {
                    EpisodeNumberDecision::Regular(n) => {
                        (EpisodeNumber::Regular { numero: *n }, None)
                    }
                    EpisodeNumberDecision::Special(label) => (
                        EpisodeNumber::Special {
                            label: label.clone(),
                        },
                        Some(label.clone()),
                    ),
                };

                let episode = Episode {
                    id: Uuid::new_v4(),
                    anime_id: *anime_id,
                    numero,
                    titulo,
                    duracao_esperada: None,
                    progresso_atual: 0,
                    estado: EpisodeState::NaoVisto,
                    criado_em: now,
                    atualizado_em: now,
                };

                // Validate invariants before persisting
                validate_episode(&episode)?;

                // Persist
                self.episode_repo.save(&episode)?;

                // Emit event
                let numero_str = match &episode.numero {
                    EpisodeNumber::Regular { numero } => numero.to_string(),
                    EpisodeNumber::Special { label } => label.clone(),
                };
                self.event_bus
                    .emit(EpisodeCreated::new(episode.id, *anime_id, numero_str));

                Ok((episode.id, true))
            }
            _ => Err(AppError::Other("Invalid episode decision".to_string())),
        }
    }

    /// Link a file to an episode if applicable.
    /// CORRECTED: Removed call to non-existent get_linked_files
    /// CORRECTED: Removed is_primary parameter from link_file call
    fn link_file_if_applicable(
        &self,
        episode_id: Uuid,
        file_id: Uuid,
        file_role: &str,
    ) -> AppResult<bool> {
        // Only link video and subtitle files
        if file_role != "video" && file_role != "subtitle" {
            return Ok(false);
        }

        // Check if file exists
        let file = self.file_repo.get_by_id(file_id)?;
        if file.is_none() {
            return Ok(false);
        }

        // Link file to episode (repository only accepts 2 arguments)
        self.episode_repo
            .link_file(episode_id, file_id)?;

        // Determine is_primary from file_role
        let is_primary = file_role == "video";

        // Emit event
        self.event_bus
            .emit(FileLinkedToEpisode::new(episode_id, file_id, is_primary));

        Ok(true)
    }

    // ========================================================================
    // HELPER METHODS
    // ========================================================================

    /// Find an anime by title (case-insensitive search).
    fn find_anime_by_title(&self, title: &str) -> AppResult<Option<Anime>> {
        let normalized = title.to_lowercase();

        // Get all anime and search
        // Note: In production, this should be a repository method with proper indexing
        let all_anime = self.anime_repo.list_all()?;

        for anime in all_anime {
            if anime.titulo_principal.to_lowercase() == normalized {
                return Ok(Some(anime));
            }
            for alt in &anime.titulos_alternativos {
                if alt.to_lowercase() == normalized {
                    return Ok(Some(anime));
                }
            }
        }

        Ok(None)
    }

    /// Find an episode by number within an anime.
    fn find_episode_by_number(
        &self,
        anime_id: Uuid,
        number: &EpisodeNumberDecision,
    ) -> AppResult<Option<Episode>> {
        let episodes = self.episode_repo.list_by_anime(anime_id)?;

        for episode in episodes {
            match (number, &episode.numero) {
                (EpisodeNumberDecision::Regular(n), EpisodeNumber::Regular { numero }) => {
                    if *numero == *n {
                        return Ok(Some(episode));
                    }
                }
                (
                    EpisodeNumberDecision::Special(label),
                    EpisodeNumber::Special { label: ep_label },
                ) => {
                    if ep_label.to_lowercase() == label.to_lowercase() {
                        return Ok(Some(episode));
                    }
                }
                _ => {}
            }
        }

        Ok(None)
    }

    /// Parse an episode number string into a decision.
    fn parse_episode_number(&self, number_str: &str) -> EpisodeNumberDecision {
        // Try to parse as regular number
        if let Ok(n) = number_str.parse::<u32>() {
            return EpisodeNumberDecision::Regular(n);
        }

        // Check for range (e.g., "1-3")
        if number_str.contains('-') {
            if let Some((start, _)) = number_str.split_once('-') {
                if let Ok(n) = start.parse::<u32>() {
                    return EpisodeNumberDecision::Regular(n);
                }
            }
        }

        // Treat as special
        EpisodeNumberDecision::Special(number_str.to_string())
    }

    /// Determine the final outcome based on what was created/linked.
    fn determine_outcome(
        &self,
        is_new_anime: bool,
        is_new_episode: bool,
        file_linked: bool,
    ) -> MaterializationOutcome {
        if is_new_anime {
            MaterializationOutcome::AnimeCreated
        } else if is_new_episode {
            MaterializationOutcome::EpisodeCreated
        } else if file_linked {
            MaterializationOutcome::FileLinked
        } else {
            MaterializationOutcome::EpisodeMatched
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Tests are in materialization_service_tests.rs
}
