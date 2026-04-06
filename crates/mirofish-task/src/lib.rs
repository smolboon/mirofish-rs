//! MiroFish Task - Task management with in-memory storage and SSE streaming
//!
//! Provides:
//! - Task creation, tracking, and status updates
//! - SSE progress streaming for real-time frontend updates
//! - Background task execution management

pub mod manager;
pub mod sse;

pub use manager::*;
pub use sse::*;