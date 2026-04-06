//! Graph builder service
//!
//! Handles the complete workflow of building a knowledge graph:
//! creating graph, setting ontology, adding documents, waiting for processing.

use std::time::Duration;
use tracing::{info, debug};
use tokio::time::sleep;

use mirofish_core::{AppConfig, Ontology, Result, GraphError};
use crate::client::ZepClient;

/// Service for building graphs from text documents
pub struct GraphBuilder {
    zep: ZepClient,
}

impl GraphBuilder {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            zep: ZepClient::new(config),
        }
    }

    /// Build a complete graph from text content
    pub async fn build_graph(
        &self,
        name: &str,
        ontology: &Ontology,
        text_chunks: Vec<String>,
        progress_callback: impl Fn(&str, f64),
    ) -> Result<String> {
        info!("Building graph: {}", name);

        // Step 1: Create graph
        progress_callback("Creating graph...", 0.05);
        let graph_id = self.zep.create_graph(name).await?;
        debug!("Graph created: {}", graph_id);

        // Step 2: Set ontology
        progress_callback("Setting ontology...", 0.15);
        self.zep.validate_and_set_ontology(&graph_id, ontology).await?;
        debug!("Ontology set");

        // Step 3: Add text chunks in batches
        let total_chunks = text_chunks.len();
        let batch_size = 3;
        let mut all_episodes = Vec::new();

        for (batch_idx, chunk_batch) in text_chunks.chunks(batch_size).enumerate() {
            let progress = 0.15 + (batch_idx as f64 / total_chunks as f64) * 0.40;
            progress_callback(
                &format!("Adding chunks batch {}/{}...", batch_idx + 1, (total_chunks + batch_size - 1) / batch_size),
                progress,
            );

            for chunk in chunk_batch {
                let episode = self.zep.add_document(&graph_id, chunk).await?;
                all_episodes.push(episode);
            }
        }

        // Step 4: Wait for Zep processing
        progress_callback("Waiting for Zep processing...", 0.55);
        self.wait_for_episodes(&graph_id, &all_episodes, &progress_callback).await?;

        // Step 5: Fetch graph data
        progress_callback("Fetching graph data...", 0.95);
        let graph_data = self.zep.get_graph_data(&graph_id).await?;
        info!("Graph built: graph_id={}, nodes={}, edges={}", graph_id, graph_data.node_count, graph_data.edge_count);

        progress_callback("Graph built successfully!", 1.0);
        Ok(graph_id)
    }

    async fn wait_for_episodes(
        &self,
        graph_id: &str,
        episodes: &[String],
        progress_callback: &impl Fn(&str, f64),
    ) -> Result<()> {
        let total = episodes.len();
        let mut processed = 0;
        let max_retries = 60;
        let retry_interval = Duration::from_secs(10);

        for i in 0..max_retries {
            let mut all_done = true;
            let mut current_processed = 0;

            for episode in episodes {
                match self.zep.wait_for_episode(graph_id, episode).await {
                    Ok(true) => current_processed += 1,
                    Ok(false) => all_done = false,
                    Err(e) => {
                        debug!("Error checking episode status: {}", e);
                    }
                }
            }

            processed = current_processed;
            let progress = 0.55 + (processed as f64 / total as f64) * 0.40;
            progress_callback(
                &format!("Processing {}/{} episodes...", processed, total),
                progress,
            );

            if all_done {
                return Ok(());
            }

            if i < max_retries - 1 {
                sleep(retry_interval).await;
            }
        }

        Err(GraphError::ZepApi("Timed out waiting for Zep processing".into()).into())
    }

    pub async fn delete_graph(&self, graph_id: &str) -> Result<()> {
        self.zep.delete_graph(graph_id).await
    }
}