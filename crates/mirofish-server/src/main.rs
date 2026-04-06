//! MiroFish Server - Axum-based REST API server
//!
//! Provides the backend API for the MiroFish simulation platform.
//! Serves static files for the frontend and handles all API routes.

use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use mirofish_api::{AppState, build_router};
use mirofish_core::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mirofish=debug,tower_http=info,axum=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = AppConfig::from_env().unwrap_or_else(|e| {
        tracing::warn!("Failed to load .env config, using defaults: {}", e);
        AppConfig::default()
    });

    let state = AppState::new(config);

    // Build router
    let app = build_router(state)
        // Serve static files from the frontend dist directory
        .fallback_service(
            ServeDir::new("static").fallback(ServeDir::new("static/index.html")),
        )
        // Add CORS middleware
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}