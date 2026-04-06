//! Ontology types for knowledge graph schema

use serde::{Deserialize, Serialize};

/// An entity type definition in the ontology
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityType {
    /// Entity type name (e.g., "Person", "Organization")
    pub name: String,
    /// Entity type description
    pub description: String,
    /// Key attributes for this entity type
    pub attributes: Vec<EntityAttribute>,
}

/// An attribute of an entity type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityAttribute {
    pub name: String,
    pub description: String,
    pub data_type: String, // "string", "number", "boolean", "date"
}

/// An edge/relation type definition in the ontology
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeType {
    /// Edge type name (e.g., "KNOWS", "WORKS_FOR")
    pub name: String,
    /// Edge type description
    pub description: String,
    /// Source entity types this relation can originate from
    pub source_types: Vec<String>,
    /// Target entity types this relation can point to
    pub target_types: Vec<String>,
}

/// Complete ontology definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ontology {
    /// Entity types defined in this ontology
    pub entity_types: Vec<EntityType>,
    /// Edge/relation types defined in this ontology
    pub edge_types: Vec<EdgeType>,
    /// Human-readable analysis summary of the documents
    pub analysis_summary: String,
}

impl Ontology {
    /// Create an empty ontology
    pub fn empty() -> Self {
        Self {
            entity_types: Vec::new(),
            edge_types: Vec::new(),
            analysis_summary: String::new(),
        }
    }

    /// Get entity type by name
    pub fn get_entity_type(&self, name: &str) -> Option<&EntityType> {
        self.entity_types.iter().find(|et| et.name == name)
    }

    /// Get edge type by name
    pub fn get_edge_type(&self, name: &str) -> Option<&EdgeType> {
        self.edge_types.iter().find(|et| et.name == name)
    }

    /// Get all entity type names
    pub fn entity_type_names(&self) -> Vec<&str> {
        self.entity_types.iter().map(|et| et.name.as_str()).collect()
    }

    /// Get all edge type names
    pub fn edge_type_names(&self) -> Vec<&str> {
        self.edge_types.iter().map(|et| et.name.as_str()).collect()
    }

    /// Check if ontology is valid (non-empty)
    pub fn is_valid(&self) -> bool {
        !self.entity_types.is_empty() && !self.edge_types.is_empty()
    }
}

/// Request to build a knowledge graph from documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphBuildRequest {
    /// Project ID
    pub project_id: String,
    /// Document text content
    pub document_text: String,
    /// Document filename
    pub filename: Option<String>,
}

/// Response from graph building
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphBuildResponse {
    /// Graph ID
    pub graph_id: String,
    /// Number of nodes
    pub node_count: usize,
    /// Number of edges
    pub edge_count: usize,
    /// Entity types found
    pub entity_types: Vec<String>,
    /// Ontology used
    pub ontology: Ontology,
}

/// Request to create/update ontology
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OntologyRequest {
    /// Entity types to define
    pub entity_types: Vec<EntityType>,
    /// Edge types to define
    pub edge_types: Vec<EdgeType>,
    /// Analysis summary
    pub analysis_summary: String,
}

/// Response from ontology operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OntologyResponse {
    /// Ontology created/updated
    pub ontology: Ontology,
    /// Success message
    pub message: String,
}

