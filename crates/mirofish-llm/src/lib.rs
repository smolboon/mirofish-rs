//! MiroFish LLM - LLM client and prompt management
//!
//! Provides:
//! - LLM client using async-openai (OpenAI compatible API)
//! - Prompt template management using askama
//! - Structured JSON response parsing

pub mod client;
pub mod prompts;
pub mod parsing;

pub use client::*;
pub use prompts::*;
pub use parsing::*;