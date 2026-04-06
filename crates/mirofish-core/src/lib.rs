//! MiroFish Core - Domain types, enums, and traits.
//!
//! This crate contains all the core domain types used across the MiroFish system.
//! It has minimal dependencies and is the foundation of the workspace.

pub mod project;
pub mod simulation;
pub mod report;
pub mod ontology;
pub mod task;
pub mod config;
pub mod error;

pub use project::*;
pub use simulation::*;
pub use report::*;
pub use ontology::*;
pub use task::*;
pub use config::*;
pub use error::*;

/// Result type alias for MiroFish operations
pub type Result<T> = std::result::Result<T, MiroFishError>;
