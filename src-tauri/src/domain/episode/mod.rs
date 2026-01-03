pub mod entity;
pub mod invariants;

pub use entity::{Episode, EpisodeNumber, EpisodeState};
pub use invariants::validate_episode;