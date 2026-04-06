//! Simulation types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Simulation status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
pub enum SimulationStatus {
    /// Simulation created, waiting for preparation
    Created,
    /// Preparation in progress (generating profiles, config)
    Preparing,
    /// Preparation complete, ready to run
    Ready,
    /// Simulation running
    Running,
    /// Simulation paused
    Paused,
    /// Simulation completed
    Completed,
    /// Simulation failed
    Failed,
}

/// Platform type for simulation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
pub enum SimulationPlatform {
    Twitter,
    Reddit,
    Parallel, // Both Twitter and Reddit
}

/// Agent profile for simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProfile {
    pub agent_id: usize,
    pub name: String,
    pub username: String,
    pub bio: String,
    pub persona: Persona,
    pub demographics: Demographics,
    pub activity_pattern: ActivityPattern,
    pub initial_beliefs: Vec<String>,
    pub social_network: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Persona {
    pub personality_traits: HashMap<String, String>,
    pub interests: Vec<String>,
    pub behavioral_tendencies: HashMap<String, String>,
    pub communication_style: String,
    pub stance_on_topic: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Demographics {
    pub age_group: String,
    pub gender: String,
    pub occupation: String,
    pub location: String,
    pub education: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityPattern {
    pub activity_level: String, // "low", "medium", "high"
    pub posting_frequency: String,
    pub peak_hours: Vec<usize>,
    pub preferred_topics: Vec<String>,
}

/// Simulation configuration (LLM-generated)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub time_config: TimeConfig,
    pub agent_configs: Vec<AgentConfig>,
    pub event_config: EventConfig,
    pub platform_config: PlatformConfig,
    pub generation_reasoning: String,
    pub generated_at: Option<DateTime<Utc>>,
    pub llm_model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeConfig {
    pub total_simulation_hours: u32,
    pub minutes_per_round: u32,
    pub peak_hours: Vec<usize>,
    pub off_peak_hours: Vec<usize>,
    pub peak_activity_multiplier: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub agent_id: usize,
    pub activity_level: f64,
    pub posting_probability: f64,
    pub comment_probability: f64,
    pub like_probability: f64,
    pub stance: String,
    pub stance_strength: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventConfig {
    pub initial_posts: Vec<InitialPost>,
    pub hot_topics: Vec<HotTopic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitialPost {
    pub content: String,
    pub topic: String,
    pub sentiment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotTopic {
    pub title: String,
    pub description: String,
    pub related_entities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
    pub twitter_config: Option<PlatformDetailConfig>,
    pub reddit_config: Option<PlatformDetailConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformDetailConfig {
    pub agent_count: usize,
    pub subreddit: String,
    pub topic: String,
}

/// Simulation state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Simulation {
    pub simulation_id: String,
    pub project_id: String,
    pub graph_id: Option<String>,
    pub status: SimulationStatus,
    pub enable_twitter: bool,
    pub enable_reddit: bool,
    pub entities_count: Option<usize>,
    pub entity_types: Vec<String>,
    pub profiles_count: Option<usize>,
    pub config_generated: bool,
    pub current_round: u32,
    pub total_rounds: u32,
    pub error: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl AgentProfile {
    /// Create a default agent profile for interview purposes
    pub fn default_for_interview() -> Self {
        Self {
            agent_id: 0,
            name: "Default Agent".to_string(),
            username: "@default".to_string(),
            bio: "A simulated agent".to_string(),
            persona: Persona {
                personality_traits: HashMap::new(),
                interests: vec![],
                behavioral_tendencies: HashMap::new(),
                communication_style: "neutral".to_string(),
                stance_on_topic: "open".to_string(),
            },
            demographics: Demographics {
                age_group: "unknown".to_string(),
                gender: "unknown".to_string(),
                occupation: "unknown".to_string(),
                location: "unknown".to_string(),
                education: "unknown".to_string(),
            },
            activity_pattern: ActivityPattern {
                activity_level: "medium".to_string(),
                posting_frequency: "moderate".to_string(),
                peak_hours: vec![],
                preferred_topics: vec![],
            },
            initial_beliefs: vec![],
            social_network: vec![],
        }
    }
}

impl Simulation {
    pub fn new(project_id: &str, graph_id: &str, enable_twitter: bool, enable_reddit: bool) -> Self {
        let now = Utc::now();
        Self {
            simulation_id: format!("sim_{}", uuid::Uuid::new_v4().simple()),
            project_id: project_id.to_string(),
            graph_id: Some(graph_id.to_string()),
            status: SimulationStatus::Created,
            enable_twitter,
            enable_reddit,
            entities_count: None,
            entity_types: Vec::new(),
            profiles_count: None,
            config_generated: false,
            current_round: 0,
            total_rounds: 0,
            error: None,
            started_at: None,
            completed_at: None,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Simulation run state (real-time tracking)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunState {
    pub simulation_id: String,
    pub runner_status: RunnerStatus,
    pub current_round: u32,
    pub total_rounds: u32,
    pub simulated_hours: u32,
    pub total_simulation_hours: u32,
    pub progress_percent: f64,
    pub twitter_running: bool,
    pub reddit_running: bool,
    pub twitter_actions_count: usize,
    pub reddit_actions_count: usize,
    pub total_actions_count: usize,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
pub enum RunnerStatus {
    Idle,
    Running,
    Paused,
    WaitingCommand, // Interview mode
    Stopped,
    Completed,
    Failed,
}

/// Agent action in simulation (matching OASIS)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentAction {
    pub round_num: u32,
    pub timestamp: DateTime<Utc>,
    pub platform: String, // "twitter" or "reddit"
    pub agent_id: usize,
    pub agent_name: String,
    pub action_type: String,
    pub action_args: serde_json::Value,
    pub result: Option<String>,
    pub success: bool,
}

/// Request to create a new simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSimulationRequest {
    /// Project ID
    pub project_id: String,
    /// Graph ID
    pub graph_id: String,
    /// Enable Twitter platform
    pub enable_twitter: bool,
    /// Enable Reddit platform
    pub enable_reddit: bool,
    /// Simulation requirements/description
    pub simulation_requirement: Option<String>,
}

/// Request to prepare a simulation (generate profiles, config)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrepareSimulationRequest {
    /// Simulation ID
    pub simulation_id: String,
    /// Graph ID
    pub graph_id: String,
    /// Document text for context
    pub document_text: String,
    /// Simulation requirements/description
    pub simulation_requirement: Option<String>,
    /// Enable Twitter platform
    pub enable_twitter: bool,
    /// Enable Reddit platform
    pub enable_reddit: bool,
    /// LLM model to use
    pub llm_model: Option<String>,
}

/// Request to start a simulation run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartSimulationRequest {
    /// Simulation ID
    pub simulation_id: String,
    /// Simulation config (generated during prepare)
    pub simulation_config: SimulationConfig,
    /// Agent profiles (generated during prepare)
    pub profiles: Vec<AgentProfile>,
}

/// Response with simulation status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationStatusResponse {
    /// Simulation ID
    pub simulation_id: String,
    /// Current status
    pub status: SimulationStatus,
    /// Current round
    pub current_round: u32,
    /// Total rounds
    pub total_rounds: u32,
    /// Error message if failed
    pub error: Option<String>,
    /// Started at
    pub started_at: Option<DateTime<Utc>>,
    /// Completed at
    pub completed_at: Option<DateTime<Utc>>,
}
