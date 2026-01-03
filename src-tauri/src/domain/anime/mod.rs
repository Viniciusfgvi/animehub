pub mod entity;
pub mod invariants;

pub use entity::{Anime, AnimeType, AnimeStatus};
pub use invariants::validate_anime;