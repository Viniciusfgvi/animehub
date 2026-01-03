// src-tauri/src/services/statistics_service.rs
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;
use crate::domain::statistics::{
    StatisticsSnapshot, StatisticsType, GlobalStatistics, AnimeStatistics
};
use crate::domain::episode::EpisodeState;
use crate::repositories::{StatisticsRepository, AnimeRepository, EpisodeRepository};
use crate::events::{EventBus, StatisticsUpdated, EpisodeCompleted};
use crate::error::AppResult;

pub struct StatisticsService {
    statistics_repo: Arc<dyn StatisticsRepository>,
    anime_repo: Arc<dyn AnimeRepository>,
    episode_repo: Arc<dyn EpisodeRepository>,
    event_bus: Arc<EventBus>,
}

impl StatisticsService {
    pub fn new(
        statistics_repo: Arc<dyn StatisticsRepository>,
        anime_repo: Arc<dyn AnimeRepository>,
        episode_repo: Arc<dyn EpisodeRepository>,
        event_bus: Arc<EventBus>,
    ) -> Self {
        Self {
            statistics_repo,
            anime_repo,
            episode_repo,
            event_bus,
        }
    }

    pub fn calculate_global_statistics(&self) -> AppResult<GlobalStatistics> {
        let animes = self.anime_repo.list_all()?;
        let mut total_episodes = 0;
        let mut episodes_assistidos = 0;
        let mut tempo_total_assistido = 0u64;
        let mut animes_em_progresso = 0;
        let mut animes_completos = 0;

        for anime in &animes {
            let episodes = self.episode_repo.list_by_anime(anime.id)?;
            let mut anime_has_progress = false;
            let mut all_episodes_done = !episodes.is_empty();

            for ep in &episodes {
                total_episodes += 1;
                tempo_total_assistido += ep.progresso_atual;

                if ep.estado == EpisodeState::Concluido {
                    episodes_assistidos += 1;
                    anime_has_progress = true;
                } else if ep.progresso_atual > 0 {
                    anime_has_progress = true;
                    all_episodes_done = false;
                } else {
                    all_episodes_done = false;
                }
            }

            if all_episodes_done {
                animes_completos += 1;
            } else if anime_has_progress {
                animes_em_progresso += 1;
            }
        }

        let stats = GlobalStatistics {
            total_animes: animes.len() as u32,
            total_episodes,
            episodes_assistidos,
            tempo_total_assistido,
            animes_em_progresso,
            animes_completos,
        };

        let snapshot = StatisticsSnapshot::new(
            StatisticsType::Global,
            serde_json::to_value(&stats)?,
        );

        self.statistics_repo.save_snapshot(&snapshot)?;
        Ok(stats)
    }

    pub fn register_event_handlers(&self) {
        let episode_repo = Arc::clone(&self.episode_repo);
        let statistics_repo = Arc::clone(&self.statistics_repo);
        let event_bus = Arc::clone(&self.event_bus);

        self.event_bus.subscribe::<EpisodeCompleted, _>(move |event| {
            if let Ok(episodes) = episode_repo.list_by_anime(event.anime_id) {
                let total_episodes = episodes.len() as u32;
                let mut episodes_assistidos = 0;
                let mut tempo_assistido = 0u64;

                for ep in &episodes {
                    if ep.estado == EpisodeState::Concluido {
                        episodes_assistidos += 1;
                    }
                    tempo_assistido += ep.progresso_atual;
                }

                let progresso_percentual = if total_episodes > 0 {
                    (episodes_assistidos as f32 / total_episodes as f32) * 100.0
                } else {
                    0.0
                };

                let stats = AnimeStatistics {
                    anime_id: event.anime_id,
                    total_episodes,
                    episodes_assistidos,
                    tempo_assistido,
                    progresso_percentual,
                    ultimo_episodio_assistido: Some(event.episode_id),
                    data_ultima_visualizacao: Some(Utc::now()),
                };

                let snapshot = StatisticsSnapshot::new(
                    StatisticsType::PorAnime { anime_id: event.anime_id },
                    serde_json::to_value(&stats).unwrap(),
                );

                let _ = statistics_repo.save_snapshot(&snapshot);
            }
            event_bus.emit(StatisticsUpdated::new());
        });
    }
}