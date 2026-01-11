// src-tauri/src/services/anime_service.rs
use crate::domain::anime::{validate_anime, Anime, AnimeStatus, AnimeType};
use crate::error::{AppError, AppResult};
use crate::events::{AnimeCreated, AnimeMerged, AnimeUpdated, EventBus};
use crate::repositories::{AnimeAliasRepository, AnimeRepository, ExternalReferenceRepository};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CreateAnimeRequest {
    pub titulo_principal: String,
    pub titulos_alternativos: Vec<String>,
    pub tipo: AnimeType,
    pub status: AnimeStatus,
    pub total_episodios: Option<u32>,
    pub data_inicio: Option<DateTime<Utc>>,
    pub data_fim: Option<DateTime<Utc>>,
    pub metadados_livres: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct UpdateAnimeRequest {
    pub anime_id: Uuid,
    pub titulo_principal: Option<String>,
    pub titulos_alternativos: Option<Vec<String>>,
    pub tipo: Option<AnimeType>,
    pub status: Option<AnimeStatus>,
    pub total_episodios: Option<Option<u32>>,
    pub data_inicio: Option<Option<DateTime<Utc>>>,
    pub data_fim: Option<Option<DateTime<Utc>>>,
    pub metadados_livres: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct MergeAnimesRequest {
    pub principal_anime_id: Uuid,
    pub anime_to_merge_id: Uuid,
}

pub struct AnimeService {
    anime_repo: Arc<dyn AnimeRepository>,
    alias_repo: Arc<dyn AnimeAliasRepository>,
    external_ref_repo: Arc<dyn ExternalReferenceRepository>,
    event_bus: Arc<EventBus>,
}

impl AnimeService {
    pub fn new(
        anime_repo: Arc<dyn AnimeRepository>,
        alias_repo: Arc<dyn AnimeAliasRepository>,
        external_ref_repo: Arc<dyn ExternalReferenceRepository>,
        event_bus: Arc<EventBus>,
    ) -> Self {
        Self {
            anime_repo,
            alias_repo,
            external_ref_repo,
            event_bus,
        }
    }

    pub fn create_anime(&self, request: CreateAnimeRequest) -> AppResult<Uuid> {
        let mut anime = Anime::new(request.titulo_principal, request.tipo);

        anime.update_metadata(
            None,
            Some(request.titulos_alternativos),
            None,
            Some(request.status),
            Some(request.total_episodios),
            Some(request.data_inicio),
            Some(request.data_fim),
            Some(request.metadados_livres),
        );

        validate_anime(&anime).map_err(AppError::Domain)?;
        self.anime_repo.save(&anime)?;

        self.event_bus.emit(AnimeCreated::new(
            anime.id,
            anime.titulo_principal.clone(),
            anime.tipo.to_string(),
        ));

        Ok(anime.id)
    }

    pub fn update_anime(&self, request: UpdateAnimeRequest) -> AppResult<()> {
        let mut anime = self
            .anime_repo
            .get_by_id(request.anime_id)?
            .ok_or(AppError::NotFound)?;

        anime.update_metadata(
            request.titulo_principal,
            request.titulos_alternativos,
            request.tipo,
            request.status,
            request.total_episodios,
            request.data_inicio,
            request.data_fim,
            request.metadados_livres,
        );

        validate_anime(&anime).map_err(AppError::Domain)?;
        self.anime_repo.save(&anime)?;

        self.event_bus.emit(AnimeUpdated::new(anime.id));
        Ok(())
    }

    pub fn get_anime(&self, anime_id: Uuid) -> AppResult<Option<Anime>> {
        self.anime_repo.get_by_id(anime_id)
    }

    pub fn list_all_animes(&self) -> AppResult<Vec<Anime>> {
        self.anime_repo.list_all()
    }

    pub fn merge_animes(&self, request: MergeAnimesRequest) -> AppResult<()> {
        let principal = self
            .anime_repo
            .get_by_id(request.principal_anime_id)?
            .ok_or(AppError::NotFound)?;
        let to_merge = self
            .anime_repo
            .get_by_id(request.anime_to_merge_id)?
            .ok_or(AppError::NotFound)?;

        let alias =
            crate::domain::AnimeAlias::new(principal.id, to_merge.id).map_err(AppError::Other)?;

        self.alias_repo.save(&alias)?;

        self.event_bus.emit(AnimeMerged::new(
            request.principal_anime_id,
            request.anime_to_merge_id,
        ));

        Ok(())
    }

    pub fn resolve_alias(&self, anime_id: Uuid) -> AppResult<Uuid> {
        if let Some(principal_id) = self.alias_repo.get_principal_for_alias(anime_id)? {
            Ok(principal_id)
        } else {
            Ok(anime_id)
        }
    }

    pub fn get_external_references(
        &self,
        anime_id: Uuid,
    ) -> AppResult<Vec<crate::domain::ExternalReference>> {
        self.external_ref_repo.list_by_anime(anime_id)
    }
}
