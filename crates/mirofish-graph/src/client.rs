//! Zep Cloud HTTP client

use std::sync::Arc;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::debug;

use mirofish_core::{AppConfig, GraphError, Result, Ontology};

/// Zep Cloud API client
#[derive(Clone)]
pub struct ZepClient {
    client: Arc<Client>,
    base_url: String,
}

// Ensure ZepClient is Send + Sync + 'static for axum State
static_assertions::assert_impl_all!(ZepClient: Send, Sync);

impl ZepClient {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            client: Arc::new(Client::builder()
                .default_headers({
                    let mut h = reqwest::header::HeaderMap::new();
                    h.insert("Authorization", 
                        reqwest::header::HeaderValue::from_str(&format!("ApiKey {}", config.zep_api_key)).unwrap());
                    h.insert("Content-Type", 
                        reqwest::header::HeaderValue::from_static("application/json"));
                    h
                })
                .build()
                .unwrap()),
            base_url: config.zep_base_url.clone(),
        }
    }

    pub async fn create_graph(&self, name: &str) -> Result<String> {
        debug!("Creating Zep graph: {}", name);
        let resp = self.client
            .post(format!("{}/api/v2/graphs", self.base_url))
            .json(&serde_json::json!({"name": name}))
            .send()
            .await
            .map_err(|e| GraphError::ZepApi(e.to_string()))?;
        
        let body: serde_json::Value = resp.json().await
            .map_err(|e| GraphError::ZepApi(e.to_string()))?;
        
        body.get("uuid")
            .and_then(|v| v.as_str())
            .map(String::from)
            .ok_or_else(|| GraphError::ZepApi("No UUID in response".into()).into())
    }

    pub async fn set_ontology(&self, graph_id: &str, ontology: &Ontology) -> Result<()> {
        debug!("Setting ontology for graph {}", graph_id);
        // Convert to Zep schema format
        let schema = self.ontology_to_zep_schema(ontology);
        self.client
            .post(format!("{}/api/v2/graphs/{}/schema", self.base_url, graph_id))
            .json(&schema)
            .send()
            .await
            .map_err(|e| GraphError::ZepApi(e.to_string()))?;
        Ok(())
    }

    pub async fn add_document(&self, graph_id: &str, text: &str) -> Result<String> {
        debug!("Adding document to graph {}", graph_id);
        let resp = self.client
            .post(format!("{}/api/v2/graphs/{}/documents", self.base_url, graph_id))
            .json(&serde_json::json!({
                "text": text,
                "metadata": {}
            }))
            .send()
            .await
            .map_err(|e| GraphError::ZepApi(e.to_string()))?;
        
        let body: serde_json::Value = resp.json().await
            .map_err(|e| GraphError::ZepApi(e.to_string()))?;
        
        body.get("uuid")
            .and_then(|v| v.as_str())
            .map(String::from)
            .ok_or_else(|| GraphError::ZepApi("No episode UUID in response".into()).into())
    }

    pub async fn wait_for_episode(&self, graph_id: &str, episode_uuid: &str) -> Result<bool> {
        let resp = self.client
            .get(format!("{}/api/v2/graphs/{}/episodes/{}", self.base_url, graph_id, episode_uuid))
            .send()
            .await
            .map_err(|e| GraphError::ZepApi(e.to_string()))?;
        
        let body: serde_json::Value = resp.json().await
            .map_err(|e| GraphError::ZepApi(e.to_string()))?;
        
        Ok(body.get("processing_complete").and_then(|v| v.as_bool()).unwrap_or(false))
    }

    pub async fn get_graph_data(&self, graph_id: &str) -> Result<GraphData> {
        let resp = self.client
            .get(format!("{}/api/v2/graphs/{}/graph", self.base_url, graph_id))
            .send()
            .await
            .map_err(|e| GraphError::ZepApi(e.to_string()))?;
        
        let body: serde_json::Value = resp.json().await
            .map_err(|e| GraphError::ZepApi(e.to_string()))?;
        
        let nodes: Vec<GraphNode> = serde_json::from_value(
            body.get("nodes").cloned().unwrap_or_default()
        ).map_err(|e| GraphError::ZepApi(e.to_string()))?;
        
        let edges: Vec<GraphEdge> = serde_json::from_value(
            body.get("edges").cloned().unwrap_or_default()
        ).map_err(|e| GraphError::ZepApi(e.to_string()))?;

        let node_count = nodes.len();
        let edge_count = edges.len();

        Ok(GraphData {
            nodes,
            edges,
            node_count,
            edge_count,
        })
    }

    pub async fn delete_graph(&self, graph_id: &str) -> Result<()> {
        self.client
            .delete(format!("{}/api/v2/graphs/{}", self.base_url, graph_id))
            .send()
            .await
            .map_err(|e| GraphError::ZepApi(e.to_string()))?;
        Ok(())
    }

    pub async fn get_entities(&self, graph_id: &str, entity_type: Option<&str>) -> Result<Vec<ZepEntity>> {
        let mut url = format!("{}/api/v2/graphs/{}/entities", self.base_url, graph_id);
        if let Some(et) = entity_type {
            url.push_str(&format!("?entity_type={}", et));
        }
        
        let resp = self.client.get(&url).send().await
            .map_err(|e| GraphError::ZepApi(e.to_string()))?;
        
        let body: serde_json::Value = resp.json().await
            .map_err(|e| GraphError::ZepApi(e.to_string()))?;
        
        let entities: Vec<ZepEntity> = serde_json::from_value(
            body.get("entities").cloned().unwrap_or_default()
        ).map_err(|e| GraphError::ZepApi(e.to_string()))?;
        
        Ok(entities)
    }

    pub async fn search_graph(&self, graph_id: &str, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let resp = self.client
            .post(format!("{}/api/v2/graphs/{}/search", self.base_url, graph_id))
            .json(&serde_json::json!({
                "query": query,
                "limit": limit
            }))
            .send()
            .await
            .map_err(|e| GraphError::ZepApi(e.to_string()))?;
        
        let body: serde_json::Value = resp.json().await
            .map_err(|e| GraphError::ZepApi(e.to_string()))?;
        
        let results: Vec<SearchResult> = serde_json::from_value(
            body.get("results").cloned().unwrap_or_default()
        ).map_err(|e| GraphError::ZepApi(e.to_string()))?;
        
        Ok(results)
    }

    fn ontology_to_zep_schema(&self, ontology: &Ontology) -> serde_json::Value {
        serde_json::json!({
            "entity_types": ontology.entity_types.iter().map(|et| {
                serde_json::json!({
                    "name": et.name,
                    "description": et.description,
                })
            }).collect::<Vec<_>>(),
            "edge_types": ontology.edge_types.iter().map(|et| {
                serde_json::json!({
                    "name": et.name,
                    "description": et.description,
                    "source_types": et.source_types,
                    "target_types": et.target_types,
                })
            }).collect::<Vec<_>>(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub node_count: usize,
    pub edge_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub uuid: String,
    pub name: String,
    pub entity_type: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub uuid: String,
    pub name: String,
    pub source_node_uuid: String,
    pub target_node_uuid: String,
    pub facts: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZepEntity {
    pub uuid: String,
    pub name: String,
    pub entity_type: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub entity: Option<ZepEntity>,
    pub facts: Option<Vec<String>>,
    pub similarity_score: Option<f64>,
}