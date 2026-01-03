use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a single episode belonging to an Anime
/// Episodes are the unit of viewing progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    /// Internal immutable identifier
    pub id: Uuid,
    
    /// Reference to parent Anime (REQUIRED)
    pub anime_id: Uuid,
    
    /// Episode number (regular or special)
    pub numero: EpisodeNumber,
    
    /// Episode title (optional)
    pub titulo: Option<String>,
    
    /// Expected duration in seconds (optional)
    pub duracao_esperada: Option<u64>,
    
    /// Current playback progress in seconds
    pub progresso_atual: u64,
    
    /// Viewing state
    pub estado: EpisodeState,
    
    /// Creation timestamp
    pub criado_em: DateTime<Utc>,
    
    /// Last update timestamp
    pub atualizado_em: DateTime<Utc>,
}

/// Episode number can be regular (1, 2, 3...) or special (OVA, Special)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EpisodeNumber {
    Regular { numero: u32 },
    Special { label: String },
}

/// Current viewing state of an episode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EpisodeState {
    NaoVisto,
    EmProgresso,
    Concluido,
}

impl Episode {
    /// Create a new Episode
    /// anime_id MUST be valid (checked by caller)
    pub fn new(anime_id: Uuid, numero: EpisodeNumber) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            anime_id,
            numero,
            titulo: None,
            duracao_esperada: None,
            progresso_atual: 0,
            estado: EpisodeState::NaoVisto,
            criado_em: now,
            atualizado_em: now,
        }
    }
    
    /// Update episode metadata
    pub fn update_metadata(
        &mut self,
        titulo: Option<String>,
        duracao_esperada: Option<Option<u64>>,
    ) {
        if let Some(t) = titulo {
            self.titulo = Some(t);
        }
        if let Some(d) = duracao_esperada {
            self.duracao_esperada = d;
        }
        self.atualizado_em = Utc::now();
    }
    
    /// Update progress
    /// Returns error if progress violates invariants
    pub fn update_progress(&mut self, progresso: u64) -> Result<(), String> {
        // Progress cannot exceed duration (if known)
        if let Some(duracao) = self.duracao_esperada {
            if progresso > duracao {
                return Err(format!(
                    "Progress {} exceeds duration {}", 
                    progresso, 
                    duracao
                ));
            }
        }
        
        // Progress should not decrease (except explicit reset)
        if progresso < self.progresso_atual && progresso != 0 {
            // Allow reset to 0, but not arbitrary decreases
            return Err(format!(
                "Progress cannot decrease from {} to {} (use reset if intentional)",
                self.progresso_atual,
                progresso
            ));
        }
        
        self.progresso_atual = progresso;
        
        // Update state based on progress
        self.estado = if progresso == 0 {
            EpisodeState::NaoVisto
        } else if let Some(duracao) = self.duracao_esperada {
            if progresso >= (duracao * 90 / 100) {
                EpisodeState::Concluido
            } else {
                EpisodeState::EmProgresso
            }
        } else {
            EpisodeState::EmProgresso
        };
        
        self.atualizado_em = Utc::now();
        Ok(())
    }
    
    /// Mark as completed
    pub fn mark_completed(&mut self) {
        self.estado = EpisodeState::Concluido;
        if let Some(duracao) = self.duracao_esperada {
            self.progresso_atual = duracao;
        }
        self.atualizado_em = Utc::now();
    }
    
    /// Reset progress
    pub fn reset_progress(&mut self) {
        self.progresso_atual = 0;
        self.estado = EpisodeState::NaoVisto;
        self.atualizado_em = Utc::now();
    }
}

impl EpisodeNumber {
    pub fn regular(numero: u32) -> Self {
        Self::Regular { numero }
    }
    
    pub fn special(label: String) -> Self {
        Self::Special { label }
    }
}

impl std::fmt::Display for EpisodeNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EpisodeNumber::Regular { numero } => write!(f, "{}", numero),
            EpisodeNumber::Special { label } => write!(f, "{}", label),
        }
    }
}

impl std::fmt::Display for EpisodeState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EpisodeState::NaoVisto => write!(f, "nao_visto"),
            EpisodeState::EmProgresso => write!(f, "em_progresso"),
            EpisodeState::Concluido => write!(f, "concluido"),
        }
    }
}