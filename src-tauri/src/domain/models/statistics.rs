use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;
use std::str::FromStr;
use crate::shared::errors::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatisticsType {
    Global,
    Genre(String),
    Status(String),
    Year(i32),
}

impl std::fmt::Display for StatisticsType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Global => write!(f, "global"),
            Self::Genre(g) => write!(f, "genre:{}", g),
            Self::Status(s) => write!(f, "status:{}", s),
            Self::Year(y) => write!(f, "year:{}", y),
        }
    }
}

// Melhoria Arquitetural: Lógica de parsing extraída do repositório para o domínio
impl FromStr for StatisticsType {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "global" {
            Ok(Self::Global)
        } else if let Some(genre) = s.strip_prefix("genre:") {
            Ok(Self::Genre(genre.to_string()))
        } else if let Some(status) = s.strip_prefix("status:") {
            Ok(Self::Status(status.to_string()))
        } else if let Some(year_str) = s.strip_prefix("year:") {
            let year = year_str.parse::<i32>()
                .map_err(|_| AppError::Validation("Invalid year format in statistics type".into()))?;
            Ok(Self::Year(year))
        } else {
            Err(AppError::Validation(format!("Unknown statistics type: {}", s)))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticsSnapshot {
    pub id: Uuid,
    pub tipo: StatisticsType,
    pub data: HashMap<String, f64>,
    pub captured_at: i64,
}