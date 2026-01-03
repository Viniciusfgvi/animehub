use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a user-defined collection of anime
/// Collections are purely organizational and do not affect anime state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    /// Internal immutable identifier
    pub id: Uuid,
    
    /// Collection name
    pub nome: String,
    
    /// Optional description
    pub descricao: Option<String>,
    
    /// Creation timestamp
    pub criado_em: DateTime<Utc>,
}

impl Collection {
    /// Create a new Collection
    pub fn new(nome: String, descricao: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            nome,
            descricao,
            criado_em: Utc::now(),
        }
    }
    
    /// Update collection metadata
    pub fn update(&mut self, nome: Option<String>, descricao: Option<Option<String>>) {
        if let Some(n) = nome {
            self.nome = n;
        }
        if let Some(d) = descricao {
            self.descricao = d;
        }
    }
}

impl std::fmt::Display for Collection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.nome)
    }
}