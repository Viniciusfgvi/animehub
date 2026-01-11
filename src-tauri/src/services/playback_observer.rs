// src-tauri/src/services/playback_observer.rs
//
// Playback Observer - Background monitoring of MPV state
//
// CRITICAL RULES:
// - Runs in background task
// - Polls MPV periodically for position, pause state, eof-reached
// - Emits domain events based on observed changes
// - Does NOT persist state directly
// - Does NOT call domain services

use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::task::JoinHandle;
use uuid::Uuid;

use crate::events::types::{PlaybackFinished, PlaybackProgressUpdated, PlaybackStopped};
use crate::events::EventBus;
use crate::integrations::mpv::MpvClient;

#[derive(Debug, Clone)]
pub struct ObserverConfig {
    pub poll_interval_ms: u64,
    pub min_progress_delta: u64,
    pub completion_threshold: f32,
}

impl Default for ObserverConfig {
    fn default() -> Self {
        Self {
            poll_interval_ms: 2000,
            min_progress_delta: 5,
            completion_threshold: 0.90,
        }
    }
}

#[derive(Clone)]
struct ObservationSession {
    episode_id: Uuid,
    last_reported_position: u64,
    duration: Option<u64>,
    completed_emitted: bool,
}

pub struct PlaybackObserver {
    mpv_client: Arc<MpvClient>,
    event_bus: Arc<EventBus>,
    config: ObserverConfig,
    current_session: Arc<Mutex<Option<ObservationSession>>>,
    task_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl PlaybackObserver {
    pub fn new(
        mpv_client: Arc<MpvClient>,
        event_bus: Arc<EventBus>,
        config: ObserverConfig,
    ) -> Self {
        Self {
            mpv_client,
            event_bus,
            config,
            current_session: Arc::new(Mutex::new(None)),
            task_handle: Arc::new(Mutex::new(None)),
        }
    }

    pub fn start_observing(&self, episode_id: Uuid, initial_position: u64, duration: Option<u64>) {
        self.stop_observing();

        {
            let mut session = self.current_session.lock().unwrap();
            *session = Some(ObservationSession {
                episode_id,
                last_reported_position: initial_position,
                duration,
                completed_emitted: false,
            });
        }

        self.spawn_observer_task();
    }

    pub fn stop_observing(&self) {
        {
            let mut session = self.current_session.lock().unwrap();
            *session = None;
        }
        let mut handle = self.task_handle.lock().unwrap();
        if let Some(task) = handle.take() {
            task.abort();
        }
    }

    pub fn is_observing(&self) -> bool {
        self.current_session.lock().unwrap().is_some()
    }

    fn spawn_observer_task(&self) {
        let mpv_client = Arc::clone(&self.mpv_client);
        let event_bus = Arc::clone(&self.event_bus);
        let session = Arc::clone(&self.current_session);
        let config = self.config.clone();

        let task = tokio::spawn(async move {
            let interval = Duration::from_millis(config.poll_interval_ms);

            loop {
                tokio::time::sleep(interval).await;

                let mut current = {
                    let guard = session.lock().unwrap();
                    if let Some(s) = guard.as_ref() {
                        s.clone()
                    } else {
                        break;
                    }
                };

                if !mpv_client.is_running() {
                    event_bus.emit(PlaybackStopped::new(
                        current.episode_id,
                        current.last_reported_position,
                    ));
                    let mut guard = session.lock().unwrap();
                    *guard = None;
                    break;
                }

                let position = match mpv_client.get_position() {
                    Ok(p) => p,
                    Err(_) => continue,
                };

                // Atualiza duração se ainda desconhecida
                if current.duration.is_none() {
                    if let Ok(Some(dur)) = mpv_client.get_duration() {
                        current.duration = Some(dur);
                    }
                }

                // Verifica conclusão (90% ou mais)
                let completed = current.completed_emitted
                    || current.duration.map_or(false, |dur| {
                        (position as f32 / dur as f32) >= config.completion_threshold
                    });

                if completed && !current.completed_emitted {
                    // Usa duração conhecida ou 0 como fallback
                    let duration_seconds = current.duration.unwrap_or(0);
                    event_bus.emit(PlaybackFinished::new(current.episode_id, duration_seconds));
                    current.completed_emitted = true;
                }

                // Atualiza progresso se mudou significativamente
                let delta = if position > current.last_reported_position {
                    position - current.last_reported_position
                } else {
                    current.last_reported_position - position
                };

                if delta >= config.min_progress_delta {
                    event_bus.emit(PlaybackProgressUpdated::new(current.episode_id, position));
                    current.last_reported_position = position;
                }

                // Atualiza sessão
                {
                    let mut guard = session.lock().unwrap();
                    if let Some(s) = guard.as_mut() {
                        s.last_reported_position = position;
                        s.duration = current.duration;
                        s.completed_emitted = current.completed_emitted;
                    }
                }
            }
        });

        let mut handle = self.task_handle.lock().unwrap();
        *handle = Some(task);
    }
}

impl Drop for PlaybackObserver {
    fn drop(&mut self) {
        self.stop_observing();
    }
}
