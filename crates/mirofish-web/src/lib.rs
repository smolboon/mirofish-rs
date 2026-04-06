//! MiroFish Web - Wasm frontend
//!
//! Provides the WebAssembly-based frontend using:
//! - Leptos for reactive UI components
//! - leptos_router for client-side routing
//! - gloo-net for API communication
//! - wasm-bindgen for JS interop
//! - Force-directed graph visualization
//! - Internationalization (i18n) support

pub mod api;
pub mod i18n;
pub mod components;
pub mod app;

pub use api::*;
pub use i18n::*;
pub use components::*;
pub use app::*;
