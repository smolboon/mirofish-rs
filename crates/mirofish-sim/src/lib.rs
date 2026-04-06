//! MiroFish Simulation Engine
//!
//! Provides the multi-agent social simulation engine:
//! - Agent profiles and behavior
//! - Twitter/Reddit platform simulation
//! - Simulation orchestration
//! - Post-simulation interview mode

pub mod actions;
pub mod agent;
pub mod platform;
pub mod config;
pub mod profile;
pub mod engine;
pub mod interview;

pub use actions::*;
pub use agent::*;
pub use platform::*;
pub use config::*;
pub use profile::*;
pub use engine::*;
pub use interview::*;