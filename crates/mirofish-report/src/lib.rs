//! MiroFish Report - Report Agent with ReACT pattern
//!
//! Provides:
//! - ReportAgent with ReACT (Reasoning + Acting) mode
//! - Tool definitions (InsightForge, PanoramaSearch, QuickSearch, InterviewAgents)
//! - Report planning and outline generation
//! - Section-by-section generation with tool-augmented research
//! - Chat interface for post-report interaction

pub mod agent;
pub mod react;
pub mod tools;
pub mod planner;
pub mod chat;

pub use agent::*;
pub use react::*;
pub use tools::*;
pub use planner::*;
pub use chat::*;