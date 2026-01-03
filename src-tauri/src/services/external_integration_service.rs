// src-tauri/src/services/external_integration_service.rs
use std::sync::Arc;
use uuid::Uuid;
use crate::domain::{ExternalReference, validate_external_reference};
use crate::repositories::{ExternalReferenceRepository, AnimeRepository};
use crate::events::{EventBus, ExternalMetadataRequested, ExternalMetadataFetched, ExternalMetadataLinked};
use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalMetadata {
    pub external_id: String,
    pub title: String,
    pub alternative_titles: Vec<String>,
    pub total_episodes: Option<u32>,
    pub status: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub genres: Vec<String>,
    pub cover_image: Option<String>,
    pub synopsis: Option<String>,
}

// Esta estrutura estava faltando e causando erro no mod.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataSuggestions {
    pub provider: String,
    pub suggestions: Vec<ExternalMetadata>,
}

#[derive(Debug, Clone)]
pub struct FetchMetadataRequest {
    pub anime_id: Uuid,
    pub provider: String,
}

#[derive(Debug, Clone)]
pub struct LinkExternalReferenceRequest {
    pub anime_id: Uuid,
    pub provider: String,
    pub external_id: String,
}

pub struct ExternalIntegrationService {
    external_ref_repo: Arc<dyn ExternalReferenceRepository>,
    anime_repo: Arc<dyn AnimeRepository>,
    event_bus: Arc<EventBus>,
}

impl ExternalIntegrationService {
    pub fn new(
        external_ref_repo: Arc<dyn ExternalReferenceRepository>,
        anime_repo: Arc<dyn AnimeRepository>,
        event_bus: Arc<EventBus>,
    ) -> Self {
        Self {
            external_ref_repo,
            anime_repo,
            event_bus,
        }
    }

    pub fn fetch_and_link_metadata(&self, request: FetchMetadataRequest) -> AppResult<()> {
        self.event_bus.emit(ExternalMetadataRequested::new(
            request.anime_id,
            request.provider.clone(),
        ));
        Ok(())
    }

    pub fn link_external_reference(&self, request: LinkExternalReferenceRequest) -> AppResult<()> {
        if !self.anime_repo.exists(request.anime_id)? {
            return Err(AppError::NotFound);
        }

        let reference = ExternalReference::new(
            request.anime_id,
            request.provider.clone(),
            request.external_id.clone(),
        );

        validate_external_reference(&reference)
            .map_err(AppError::Domain)?;

        self.external_ref_repo.save(&reference)?;

        self.event_bus.emit(ExternalMetadataLinked::new(
            request.anime_id,
            request.provider,
            request.external_id,
        ));

        Ok(())
    }

    pub fn search_external(&self, _provider: &str, _query: &str) -> AppResult<Vec<ExternalMetadata>> {
        let results: Vec<ExternalMetadata> = Vec::new();
        Ok(results)
    }

    pub fn sync_metadata_from_external(
        &self,
        anime_id: Uuid,
        provider: &str,
    ) -> AppResult<ExternalMetadata> {
        let anime = self.anime_repo
            .get_by_id(anime_id)?
            .ok_or(AppError::NotFound)?;

        let reference = self.external_ref_repo
            .get_by_anime_and_source(anime_id, provider)?
            .ok_or_else(|| AppError::Other("No external reference found".to_string()))?;

        let metadata = ExternalMetadata {
            external_id: reference.external_id.clone(),
            title: anime.titulo_principal.clone(),
            alternative_titles: anime.titulos_alternativos.clone(),
            total_episodes: anime.total_episodios,
            status: Some(anime.status.to_string()),
            start_date: anime.data_inicio.map(|d| d.to_rfc3339()),
            end_date: anime.data_fim.map(|d| d.to_rfc3339()),
            genres: Vec::new(),
            cover_image: None,
            synopsis: None,
        };

        self.event_bus.emit(ExternalMetadataFetched::new(
            anime_id,
            provider.to_string(),
            reference.external_id,
        ));

        Ok(metadata)
    }
}