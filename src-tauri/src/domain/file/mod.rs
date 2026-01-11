pub mod entity;
pub mod invariants;

pub use entity::{File, FileOrigin, FileType};
pub use invariants::validate_file;
