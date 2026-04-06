//! Ontology management for Zep graphs
//!
//! Handles generating ontology from text via LLM, setting, getting,
//! and validating ontology schemas.

use mirofish_core::{EntityType as CoreEntityType, EdgeType, Ontology, GraphError, Result};
use mirofish_llm::LLMClient;
use mirofish_llm::prompts::{
    ONTOLOGY_SYSTEM_PROMPT,
    ONTOLOGY_USER_PROMPT_TEMPLATE,
};
use crate::client::ZepClient;

/// Generate ontology from text using LLM
pub async fn generate_ontology(
    llm: &LLMClient,
    text: &str,
    simulation_requirement: &str,
) -> Result<Ontology> {
    let sample_text = if text.len() > 2000 {
        &text[..2000]
    } else {
        text
    };

    let prompt = ONTOLOGY_USER_PROMPT_TEMPLATE
        .replace("{sample_text}", sample_text)
        .replace("{simulation_requirement}", simulation_requirement);

    // Use chat_json to get structured response
    let response: OntologyResponse = llm
        .chat_json(&ONTOLOGY_SYSTEM_PROMPT, &prompt)
        .await
        .map_err(|e| GraphError::Ontology(format!("LLM failed: {}", e)))?;

    // Convert response to Ontology
    let ontology = Ontology {
        entity_types: response.entity_types.into_iter().map(|et| CoreEntityType {
            name: et.name,
            description: et.description,
            attributes: et.attributes,
        }).collect(),
        edge_types: response.edge_types.into_iter().map(|et| EdgeType {
            name: et.name,
            description: et.description,
            source_types: et.source_types,
            target_types: et.target_types,
        }).collect(),
        analysis_summary: response.analysis_summary,
    };

    if !ontology.is_valid() {
        return Err(GraphError::Ontology(
            "Generated ontology is invalid: empty entity or edge types".to_string(),
        )
        .into());
    }

    Ok(ontology)
}

/// Ontology generation response from LLM
#[derive(Debug, serde::Deserialize)]
pub struct OntologyResponse {
    pub entity_types: Vec<EntityTypeResponse>,
    pub edge_types: Vec<EdgeTypeResponse>,
    pub analysis_summary: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct EntityTypeResponse {
    pub name: String,
    pub description: String,
    pub attributes: Vec<mirofish_core::EntityAttribute>,
}

#[derive(Debug, serde::Deserialize)]
pub struct EdgeTypeResponse {
    pub name: String,
    pub description: String,
    pub source_types: Vec<String>,
    pub target_types: Vec<String>,
}

impl ZepClient {
    /// Validate and set ontology on a graph
    pub async fn validate_and_set_ontology(&self, graph_id: &str, ontology: &Ontology) -> Result<()> {
        if !ontology.is_valid() {
            return Err(GraphError::Ontology("Invalid ontology: empty entity or edge types".into()).into());
        }
        self.set_ontology(graph_id, ontology).await
    }
}
