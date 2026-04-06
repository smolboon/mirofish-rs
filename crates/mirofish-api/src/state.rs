//! Shared application state for API handlers

use std::sync::Arc;

use mirofish_core::{AppConfig, Simulation};
use mirofish_graph::client::ZepClient;
use mirofish_llm::LLMClient;
use mirofish_task::TaskManager;
use mirofish_sim::engine::SimulationEngine;

use tokio::sync::RwLock;
use std::collections::HashMap;

use crate::report_store::ReportStore;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub llm: LLMClient,
    pub zep: ZepClient,
    pub task_manager: Arc<TaskManager>,
    /// Simulations (keyed by simulation_id)
    pub simulations: Arc<RwLock<HashMap<String, Simulation>>>,
    /// Active simulation engines (keyed by simulation_id)
    pub simulation_engines: Arc<RwLock<HashMap<String, SimulationEngine>>>,
    /// Report store
    pub report_store: Arc<ReportStore>,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        let config_arc = Arc::new(config);
        Self {
            llm: LLMClient::new(&config_arc),
            zep: ZepClient::new(&config_arc),
            task_manager: Arc::new(TaskManager::new()),
            simulations: Arc::new(RwLock::new(HashMap::new())),
            simulation_engines: Arc::new(RwLock::new(HashMap::new())),
            report_store: Arc::new(ReportStore::new()),
            config: config_arc,
        }
    }
}
