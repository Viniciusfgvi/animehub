use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a derived statistics snapshot
/// Statistics are NEVER a source of truth and can be recalculated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticsSnapshot {
    /// Snapshot identifier
    pub id: Uuid,
    
    /// Type of statistics
    pub tipo: StatisticsType,
    
    /// The actual data (stored as JSON for flexibility)
    pub valor: serde_json::Value,
    
    /// When this snapshot was generated
    pub gerado_em: DateTime<Utc>,
}

/// Types of statistics that can be tracked
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatisticsType {
    /// Global statistics across all anime
    Global,
    
    /// Statistics for a specific anime
    PorAnime { anime_id: Uuid },
    
    /// Statistics for a time period
    PorPeriodo { inicio: DateTime<Utc>, fim: DateTime<Utc> },
}

impl StatisticsSnapshot {
    /// Create a new statistics snapshot
    pub fn new(tipo: StatisticsType, valor: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            tipo,
            valor,
            gerado_em: Utc::now(),
        }
    }
}

/// Common global statistics structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalStatistics {
    pub total_animes: u32,
    pub total_episodes: u32,
    pub episodes_assistidos: u32,
    pub tempo_total_assistido: u64, // in seconds
    pub animes_em_progresso: u32,
    pub animes_completos: u32,
}

/// Per-anime statistics structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimeStatistics {
    pub anime_id: Uuid,
    pub total_episodes: u32,
    pub episodes_assistidos: u32,
    pub tempo_assistido: u64, // in seconds
    pub progresso_percentual: f32,
    pub ultimo_episodio_assistido: Option<Uuid>,
    pub data_ultima_visualizacao: Option<DateTime<Utc>>,
}

impl std::fmt::Display for StatisticsType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StatisticsType::Global => write!(f, "global"),
            StatisticsType::PorAnime { anime_id } => write!(f, "por_anime:{}", anime_id),
            StatisticsType::PorPeriodo { inicio, fim } => {
                write!(f, "por_periodo:{}:{}", inicio, fim)
            }
        }
    }
}