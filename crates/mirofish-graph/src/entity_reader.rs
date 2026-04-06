//! Entity reader for simulation preparation
//!
//! Reads and filters entities from Zep graphs for use in simulation.

use mirofish_core::Result;
use crate::client::{ZepClient, ZepEntity};

/// Filtered entity result
pub struct FilteredEntities {
    pub entities: Vec<ZepEntity>,
    pub filtered_count: usize,
    pub entity_types: Vec<String>,
}

impl ZepClient {
    /// Get entities by type from graph
    pub async fn get_entities_by_type(&self, graph_id: &str, entity_type: &str) -> Result<Vec<ZepEntity>> {
        self.get_entities(graph_id, Some(entity_type)).await
    }

    /// Get all defined entities (filtering out Entity-only nodes)
    pub async fn get_defined_entities(&self, graph_id: &str) -> Result<Vec<ZepEntity>> {
        self.get_entities(graph_id, None).await
    }

    /// Filter entities by specified types
    pub async fn filter_entities(
        &self,
        graph_id: &str,
        entity_types: Option<&[String]>,
    ) -> Result<FilteredEntities> {
        let all_entities = self.get_entities(graph_id, None).await?;
        let (entities, types) = match entity_types {
            Some(types) if !types.is_empty() => {
                let filtered: Vec<ZepEntity> = all_entities
                    .into_iter()
                    .filter(|e| types.contains(&e.entity_type))
                    .collect();
                let found_types: Vec<String> = filtered.iter().map(|e| e.entity_type.clone()).collect();
                (filtered, found_types)
            }
            _ => {
                let types: Vec<String> = all_entities.iter().map(|e| e.entity_type.clone()).collect();
                (all_entities, types)
            }
        };

        Ok(FilteredEntities {
            filtered_count: entities.len(),
            entity_types: types,
            entities,
        })
    }
}