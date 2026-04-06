//! MiroFish API - REST API endpoints
//!
//! Provides Axum route handlers for:
//! - Project management
//! - Graph building (ontology generation, graph construction)
//! - Simulation (create, prepare, start, stop, interview)
//! - Report generation and chat
//! - Task status and SSE streaming

pub mod state;
pub mod router;
pub mod graph;
pub mod simulation;
pub mod report;
pub mod project;
pub mod upload;
pub mod report_store;

pub use state::*;
pub use router::*;
