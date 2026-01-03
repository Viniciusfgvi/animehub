pub mod entity;
pub mod invariants;

pub use entity::{File, FileType, FileOrigin};
pub use invariants::validate_file;