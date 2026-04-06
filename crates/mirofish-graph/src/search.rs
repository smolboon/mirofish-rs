//! Search tools for report generation
//!
//! Provides three search strategies:
//! - InsightForge: Deep, multi-dimensional analysis
//! - PanoramaSearch: Broad, full-picture view
//! - QuickSearch: Lightweight, fast lookup

use mirofish_core::{Result, GraphError};
use crate::client::{ZepClient, SearchResult as ZepSearchResult};

/// Deep insight retrieval - decomposes query and searches multiple dimensions
pub async fn insight_forge(zep: &ZepClient, graph_id: &str, query: &str, limit: usize) -> Result<Vec<ZepSearchResult>> {
    zep.search_graph(graph_id, query, limit).await
}

/// Broad panorama search - gets complete picture with current and historical facts
pub async fn panorama_search(zep: &ZepClient, graph_id: &str, query: &str, limit: usize) -> Result<Vec<ZepSearchResult>> {
    let results = zep.search_graph(graph_id, query, limit).await?;
    Ok(results)
}

/// Quick lightweight search
pub async fn quick_search(zep: &ZepClient, graph_id: &str, query: &str, limit: usize) -> Result<Vec<ZepSearchResult>> {
    zep.search_graph(graph_id, query, limit).await
}