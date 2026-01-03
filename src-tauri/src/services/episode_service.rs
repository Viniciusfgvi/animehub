// src-tauri/src/services/episode_service.rs
//
// Episode Service - Episode and Progress Management
//
// CRITICAL RULES:
// - Manages episodes and viewing progress ONLY
// - Never creates Anime or Files
// - Never manipulates subtitles
// - Progress updates are explicit and validated

use std::sync::Arc;
use uuid::Uuid;

use crate::domain::episode::{Episode, EpisodeNumber, EpisodeState, validate_episode};
use crate::repositories::{EpisodeRepository, AnimeRepository, FileRepository};
use crate::events::{
    EventBus, EpisodeCreated, FileLinkedToEpisode, EpisodeBecamePlayable,
    EpisodeProgressUpdated, EpisodeCompleted, PlaybackStarted, PlaybackProgressUpdated
};
use crate::error::{AppError, AppResult};

/// Request to create a new episode
#[derive(Debug, Clone)]
pub struct CreateEpisodeRequest {
    pub anime_id: Uuid,
    pub numero: EpisodeNumber,
    pub titulo: Option<String>,
    pub duracao_esperada: Option<u64>,
}

/// Request to update episode metadata
#[derive(Debug, Clone)]
pub struct UpdateEpisodeMetadataRequest {
    pub episode_id: Uuid,
    pub titulo: Option<String>,
    pub duracao_esperada: Option<Option<u64>>,
}

/// Request to link a file to an episode
#[derive(Debug, Clone)]
pub struct LinkFileRequest {
    pub episode_id: Uuid,
    pub file_id: Uuid,
    pub is_primary: bool,
}

pub struct EpisodeService {
    episode_repo: Arc<dyn EpisodeRepository>,
    anime_repo: Arc<dyn AnimeRepository>,
    file_repo: Arc<dyn FileRepository>,
    event_bus: Arc<EventBus>,
}

impl EpisodeService {
    pub fn new(
        episode_repo: Arc<dyn EpisodeRepository>,
        anime_repo: Arc<dyn AnimeRepository>,
        file_repo: Arc<dyn FileRepository>,
        event_bus: Arc<EventBus>,
    ) -> Self {
        Self {
            episode_repo,
            anime_repo,
            file_repo,
            event_bus,
        }
    }
    
    /// Create a new episode
    /// 
    /// CRITICAL: anime_id MUST exist (validated)
    pub fn create_episode(&self, request: CreateEpisodeRequest) -> AppResult<Uuid> {
        // 1. Validate anime exists
        if !self.anime_repo.exists(request.anime_id)? {
            return Err(AppError::Other("Anime not found".to_string()));
        }
        
        // 2. Create domain entity
        let mut episode = Episode::new(request.anime_id, request.numero.clone());
        
        // 3. Apply additional metadata
        if let Some(titulo) = request.titulo {
            episode.titulo = Some(titulo);
        }
        if let Some(duracao) = request.duracao_esperada {
            episode.duracao_esperada = Some(duracao);
        }
        
        // 4. Validate domain invariants
        validate_episode(&episode)
            .map_err(|e| AppError::Domain(e))?;
        
        // 5. Persist
        self.episode_repo.save(&episode)?;
        
        // 6. Emit event
        self.event_bus.emit(EpisodeCreated::new(
            episode.id,
            episode.anime_id,
            episode.numero.to_string(),
        ));
        
        Ok(episode.id)
    }
    
    /// Update episode metadata
    pub fn update_episode_metadata(&self, request: UpdateEpisodeMetadataRequest) -> AppResult<()> {
        // 1. Load episode
        let mut episode = self.episode_repo
            .get_by_id(request.episode_id)?
            .ok_or(AppError::NotFound)?;
        
        // 2. Apply updates
        episode.update_metadata(request.titulo, request.duracao_esperada);
        
        // 3. Validate
        validate_episode(&episode)
            .map_err(|e| AppError::Domain(e))?;
        
        // 4. Persist
        self.episode_repo.save(&episode)?;
        
        Ok(())
    }
    
    /// Link a file to an episode
    /// 
    /// CRITICAL: File must exist and be appropriate type
    pub fn link_file(&self, request: LinkFileRequest) -> AppResult<()> {
        // 1. Validate episode exists
        let episode = self.episode_repo
            .get_by_id(request.episode_id)?
            .ok_or(AppError::NotFound)?;
        
        // 2. Validate file exists
        let file = self.file_repo
            .get_by_id(request.file_id)?
            .ok_or(AppError::NotFound)?;
        
        // 3. Create association
        self.episode_repo.link_file(request.episode_id, request.file_id, request.is_primary)?;
        
        // 4. Emit events
        self.event_bus.emit(FileLinkedToEpisode::new(
            request.episode_id,
            request.file_id,
            request.is_primary,
        ));
        
        // 5. If primary video file, emit EpisodeBecamePlayable
        if request.is_primary && file.tipo == crate::domain::FileType::Video {
            self.event_bus.emit(EpisodeBecamePlayable::new(episode.id));
        }
        
        Ok(())
    }
    
    /// Unlink a file from an episode
    pub fn unlink_file(&self, episode_id: Uuid, file_id: Uuid) -> AppResult<()> {
        self.episode_repo.unlink_file(episode_id, file_id)?;
        Ok(())
    }
    
    /// Update episode progress
    /// 
    /// CRITICAL: Progress is validated by domain
    /// CRITICAL: Never decreases automatically
    pub fn update_progress(&self, episode_id: Uuid, progress_seconds: u64) -> AppResult<()> {
        // 1. Load episode
        let mut episode = self.episode_repo
            .get_by_id(episode_id)?
            .ok_or(AppError::NotFound)?;
        
        // 2. Update progress (domain validates)
        episode.update_progress(progress_seconds)
            .map_err(|e| AppError::Other(e))?;
        
        // 3. Validate
        validate_episode(&episode)
            .map_err(|e| AppError::Domain(e))?;
        
        // 4. Persist
        self.episode_repo.save(&episode)?;
        
        // 5. Emit event
        self.event_bus.emit(EpisodeProgressUpdated::new(
            episode.id,
            episode.progresso_atual,
            episode.duracao_esperada,
        ));
        
        // 6. If completed, emit completion event
        if episode.estado == EpisodeState::Concluido {
            self.event_bus.emit(EpisodeCompleted::new(episode.id, episode.anime_id));
        }
        
        Ok(())
    }
    
    /// Mark episode as completed
    pub fn mark_completed(&self, episode_id: Uuid) -> AppResult<()> {
        // 1. Load episode
        let mut episode = self.episode_repo
            .get_by_id(episode_id)?
            .ok_or(AppError::NotFound)?;
        
        // 2. Mark completed
        episode.mark_completed();
        
        // 3. Persist
        self.episode_repo.save(&episode)?;
        
        // 4. Emit event
        self.event_bus.emit(EpisodeCompleted::new(episode.id, episode.anime_id));
        
        Ok(())
    }
    
    /// Reset episode progress
    pub fn reset_progress(&self, episode_id: Uuid) -> AppResult<()> {
        // 1. Load episode
        let mut episode = self.episode_repo
            .get_by_id(episode_id)?
            .ok_or(AppError::NotFound)?;
        
        // 2. Reset
        episode.reset_progress();
        
        // 3. Persist
        self.episode_repo.save(&episode)?;
        
        Ok(())
    }
    
    /// Get episode by ID
    pub fn get_episode(&self, episode_id: Uuid) -> AppResult<Option<Episode>> {
        self.episode_repo.get_by_id(episode_id)
    }
    
    /// List all episodes for an anime
    pub fn list_episodes_for_anime(&self, anime_id: Uuid) -> AppResult<Vec<Episode>> {
        self.episode_repo.list_by_anime(anime_id)
    }
    
    /// List episodes by state
    pub fn list_episodes_by_state(&self, anime_id: Uuid, state: EpisodeState) -> AppResult<Vec<Episode>> {
        self.episode_repo.list_by_state(anime_id, state)
    }
    
    /// Get linked files for an episode
    pub fn get_linked_files(&self, episode_id: Uuid) -> AppResult<Vec<(Uuid, bool)>> {
        self.episode_repo.get_linked_files(episode_id)
    }
    
    /// Setup event handlers
    /// This subscribes to playback events to update progress
    pub fn register_event_handlers(&self) {
        let episode_repo = Arc::clone(&self.episode_repo);
        let event_bus = Arc::clone(&self.event_bus);
        
        // Handle PlaybackStarted
        {
            let repo = Arc::clone(&episode_repo);
            self.event_bus.subscribe::<PlaybackStarted, _>(move |event| {
                if let Ok(Some(mut episode)) = repo.get_by_id(event.episode_id) {
                    episode.estado = EpisodeState::EmProgresso;
                    let _ = repo.save(&episode);
                }
            });
        }
        
        // Handle PlaybackProgressUpdated
        {
            let repo = Arc::clone(&episode_repo);
            let bus = Arc::clone(&event_bus);
            self.event_bus.subscribe::<PlaybackProgressUpdated, _>(move |event| {
                if let Ok(Some(mut episode)) = repo.get_by_id(event.episode_id) {
                    if let Ok(_) = episode.update_progress(event.progress_seconds) {
                        let _ = repo.save(&episode);
                        
                        // Emit progress update
                        bus.emit(EpisodeProgressUpdated::new(
                            episode.id,
                            episode.progresso_atual,
                            episode.duracao_esperada,
                        ));
                        
                        // Emit completion if applicable
                        if episode.estado == EpisodeState::Concluido {
                            bus.emit(EpisodeCompleted::new(episode.id, episode.anime_id));
                        }
                    }
                }
            });
        }
    }
}