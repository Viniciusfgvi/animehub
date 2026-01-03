//! Critical Statistics Invariants:
//! 
//! 1. Statistics are ALWAYS derived, NEVER primary
//! 2. Statistics can be recalculated at any time
//! 3. Statistics can be deleted without affecting domains
//! 4. Statistics NEVER alter domain state
//! 5. If statistics conflict with domain data, domain wins
//! 6. Statistics are snapshots in time
//! 7. Stale statistics are acceptable (eventual consistency)

pub mod entity;
pub use entity::{StatisticsSnapshot, StatisticsType, GlobalStatistics, AnimeStatistics};