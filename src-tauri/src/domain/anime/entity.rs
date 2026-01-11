use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a Japanese anime work (TV, Movie, OVA, Special)
/// This is the root entity for all anime-related data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anime {
    /// Internal immutable identifier
    pub id: Uuid,

    /// Primary title (usually Japanese)
    pub titulo_principal: String,

    /// Alternative titles (romaji, english, synonyms)
    pub titulos_alternativos: Vec<String>,

    /// Type of anime work
    pub tipo: AnimeType,

    /// Current status
    pub status: AnimeStatus,

    /// Total number of episodes (if known)
    pub total_episodios: Option<u32>,

    /// Start date (if known)
    pub data_inicio: Option<DateTime<Utc>>,

    /// End date (if known)
    pub data_fim: Option<DateTime<Utc>>,

    /// Free-form metadata (genres, studio, etc.)
    /// Stored as JSON internally
    pub metadados_livres: serde_json::Value,

    /// Creation timestamp
    pub criado_em: DateTime<Utc>,

    /// Last update timestamp
    pub atualizado_em: DateTime<Utc>,
}

/// Type of anime work
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum AnimeType {
    TV,
    Movie,
    OVA,
    Special,
}

/// Current status of the anime
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimeStatus {
    EmExibicao,
    Finalizado,
    Cancelado,
}

impl Anime {
    /// Create a new Anime entity
    /// This is the only way to construct a valid Anime
    pub fn new(titulo_principal: String, tipo: AnimeType) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            titulo_principal,
            titulos_alternativos: Vec::new(),
            tipo,
            status: AnimeStatus::EmExibicao,
            total_episodios: None,
            data_inicio: None,
            data_fim: None,
            metadados_livres: serde_json::Value::Object(serde_json::Map::new()),
            criado_em: now,
            atualizado_em: now,
        }
    }

    /// Update metadata
    /// This preserves the creation timestamp and updates the modification timestamp
    pub fn update_metadata(
        &mut self,
        titulo_principal: Option<String>,
        titulos_alternativos: Option<Vec<String>>,
        tipo: Option<AnimeType>,
        status: Option<AnimeStatus>,
        total_episodios: Option<Option<u32>>,
        data_inicio: Option<Option<DateTime<Utc>>>,
        data_fim: Option<Option<DateTime<Utc>>>,
        metadados_livres: Option<serde_json::Value>,
    ) {
        if let Some(titulo) = titulo_principal {
            self.titulo_principal = titulo;
        }
        if let Some(titulos) = titulos_alternativos {
            self.titulos_alternativos = titulos;
        }
        if let Some(t) = tipo {
            self.tipo = t;
        }
        if let Some(s) = status {
            self.status = s;
        }
        if let Some(total) = total_episodios {
            self.total_episodios = total;
        }
        if let Some(inicio) = data_inicio {
            self.data_inicio = inicio;
        }
        if let Some(fim) = data_fim {
            self.data_fim = fim;
        }
        if let Some(meta) = metadados_livres {
            self.metadados_livres = meta;
        }

        self.atualizado_em = Utc::now();
    }
}

impl std::fmt::Display for AnimeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnimeType::TV => write!(f, "TV"),
            AnimeType::Movie => write!(f, "Movie"),
            AnimeType::OVA => write!(f, "OVA"),
            AnimeType::Special => write!(f, "Special"),
        }
    }
}

impl std::fmt::Display for AnimeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnimeStatus::EmExibicao => write!(f, "em_exibicao"),
            AnimeStatus::Finalizado => write!(f, "finalizado"),
            AnimeStatus::Cancelado => write!(f, "cancelado"),
        }
    }
}
