//! Main simulation engine - orchestrates the simulation loop

use std::time::Duration;

use chrono::Utc;
use tokio::time::sleep;
use tracing::{info, debug, warn};

use mirofish_core::{
    AgentProfile, AgentConfig, SimulationConfig, Simulation, SimulationStatus,
    RunState, RunnerStatus,
};
use mirofish_llm::LLMClient;
use mirofish_task::TaskManager;

use crate::actions::{Post, Comment, ActionRecord, AgentAction};
use crate::agent::Agent;
use crate::platform::PlatformManager;

/// Simulation engine state
pub struct SimulationEngine {
    pub simulation: Simulation,
    pub config: SimulationConfig,
    pub agents: Vec<Agent>,
    pub posts: Vec<Post>,
    pub comments: Vec<Comment>,
    pub action_log: Vec<ActionRecord>,
    pub platform_manager: PlatformManager,
    pub current_round: u32,
    pub is_running: bool,
    pub is_paused: bool,
    pub total_actions: usize,
}

impl SimulationEngine {
    pub fn new(
        simulation: Simulation,
        config: SimulationConfig,
        profiles: Vec<AgentProfile>,
    ) -> Self {
        let agents = profiles
            .into_iter()
            .enumerate()
            .map(|(i, profile)| {
                let agent_config = if i < config.agent_configs.len() {
                    config.agent_configs[i].clone()
                } else {
                    AgentConfig {
                        agent_id: i,
                        activity_level: 0.5,
                        posting_probability: 0.3,
                        comment_probability: 0.4,
                        like_probability: 0.5,
                        stance: "neutral".to_string(),
                        stance_strength: 0.5,
                    }
                };
                Agent::new(profile, agent_config)
            })
            .collect();

        Self {
            simulation,
            config,
            agents,
            posts: Vec::new(),
            comments: Vec::new(),
            action_log: Vec::new(),
            platform_manager: PlatformManager::new(),
            current_round: 0,
            is_running: false,
            is_paused: false,
            total_actions: 0,
        }
    }

    /// Run the simulation
    pub async fn run(
        &mut self,
        llm: &LLMClient,
        task_manager: &TaskManager,
        task_id: &str,
    ) -> Result<(), String> {
        info!("Starting simulation: {}", self.simulation.simulation_id);
        self.is_running = true;
        self.simulation.status = SimulationStatus::Running;
        self.simulation.started_at = Some(Utc::now());

        let total_rounds = self.config.time_config.total_simulation_hours * 60
            / self.config.time_config.minutes_per_round;

        // Seed initial posts
        self.seed_initial_posts();

        // Main simulation loop
        for round in 1..=total_rounds {
            if !self.is_running {
                break;
            }

            // Handle pause
            while self.is_paused && self.is_running {
                sleep(Duration::from_millis(500)).await;
            }

            self.current_round = round;
            let hour = self.round_to_hour(round);

            debug!("Round {}/{} (hour {})", round, total_rounds, hour);

            // Update task progress
            let progress = ((round as f64 / total_rounds as f64) * 100.0) as u8;
            task_manager.update_task(
                task_id,
                None,
                Some(progress),
                Some(&format!("Round {}/{} (hour {})", round, total_rounds, hour)),
                Some(serde_json::json!({
                    "round": round,
                    "total_rounds": total_rounds,
                    "hour": hour,
                    "posts": self.posts.len(),
                    "comments": self.comments.len(),
                    "actions": self.total_actions,
                })),
            );

            // Process each agent - collect actions first, then execute to avoid borrow issues
            let mut agent_actions_to_execute = Vec::new();
            
            for agent_idx in 0..self.agents.len() {
                let should_act = self.agents[agent_idx].should_act(round, hour);
                if !should_act {
                    continue;
                }

                // Get recent events for context
                let recent_events = self.get_recent_events(10);

                // Decide action
                match self.agents[agent_idx]
                    .decide_next_action(
                        llm,
                        &self.posts,
                        &self.comments,
                        &recent_events,
                        round,
                    )
                    .await
                {
                    Ok(action) => {
                        if !matches!(action, AgentAction::None) {
                            agent_actions_to_execute.push((agent_idx, action));
                        }
                    }
                    Err(e) => {
                        warn!("Agent {} action failed: {}", self.agents[agent_idx].profile.name, e);
                    }
                }
            }
            
            // Execute collected actions - extract agent data first to avoid borrow issues
            for (agent_idx, action) in agent_actions_to_execute {
                let agent_id = self.agents[agent_idx].profile.agent_id;
                let agent_name = self.agents[agent_idx].profile.name.clone();
                self.execute_action_by_id(agent_id, &agent_name, action, round, hour).await;
            }

            // Small delay between rounds to avoid rate limiting
            sleep(Duration::from_millis(100)).await;
        }

        self.is_running = false;
        self.simulation.status = SimulationStatus::Completed;
        self.simulation.completed_at = Some(Utc::now());

        info!(
            "Simulation completed: {} rounds, {} posts, {} comments, {} actions",
            total_rounds,
            self.posts.len(),
            self.comments.len(),
            self.total_actions,
        );

        Ok(())
    }

    /// Pause the simulation
    pub fn pause(&mut self) {
        self.is_paused = true;
        self.simulation.status = SimulationStatus::Paused;
    }

    /// Resume the simulation
    pub fn resume(&mut self) {
        self.is_paused = false;
        self.simulation.status = SimulationStatus::Running;
    }

    /// Stop the simulation
    pub fn stop(&mut self) {
        self.is_running = false;
        self.simulation.status = SimulationStatus::Completed;
        self.simulation.completed_at = Some(Utc::now());
    }

    /// Get current run state
    pub fn get_run_state(&self) -> RunState {
        let total_rounds = self.config.time_config.total_simulation_hours * 60
            / self.config.time_config.minutes_per_round;

        RunState {
            simulation_id: self.simulation.simulation_id.clone(),
            runner_status: if self.is_paused {
                RunnerStatus::Paused
            } else if self.is_running {
                RunnerStatus::Running
            } else {
                RunnerStatus::Completed
            },
            current_round: self.current_round,
            total_rounds,
            simulated_hours: self.round_to_hour(self.current_round) as u32,
            total_simulation_hours: self.config.time_config.total_simulation_hours,
            progress_percent: if total_rounds > 0 {
                (self.current_round as f64 / total_rounds as f64) * 100.0
            } else {
                0.0
            },
            twitter_running: self.is_running && self.simulation.enable_twitter,
            reddit_running: self.is_running && self.simulation.enable_reddit,
            twitter_actions_count: self
                .action_log
                .iter()
                .filter(|a| a.platform == "twitter")
                .count(),
            reddit_actions_count: self
                .action_log
                .iter()
                .filter(|a| a.platform == "reddit")
                .count(),
            total_actions_count: self.total_actions,
            started_at: self.simulation.started_at.unwrap_or(Utc::now()),
            updated_at: Utc::now(),
        }
    }

    /// Seed initial posts from config
    fn seed_initial_posts(&mut self) {
        for (i, post_data) in self.config.event_config.initial_posts.iter().enumerate() {
            let post = Post {
                post_id: format!("post_{}", i),
                agent_id: 0,
                agent_name: "System".to_string(),
                platform: "twitter".to_string(),
                content: post_data.content.clone(),
                topic: Some(post_data.topic.clone()),
                created_at: Utc::now(),
                likes: 0,
                dislikes: 0,
                comments: Vec::new(),
                shares: 0,
            };
            self.posts.push(post);
        }
    }

    /// Convert round number to simulated hour
    fn round_to_hour(&self, round: u32) -> usize {
        let minutes_per_round = self.config.time_config.minutes_per_round;
        let total_minutes = round * minutes_per_round;
        (total_minutes / 60) as usize % 24
    }

    /// Get recent events as context strings
    fn get_recent_events(&self, limit: usize) -> Vec<String> {
        let mut events = Vec::new();

        for post in self.posts.iter().rev().take(limit / 3) {
            events.push(format!(
                "Post by {}: \"{}\"",
                post.agent_name,
                post.content.chars().take(100).collect::<String>()
            ));
        }

        for comment in self.comments.iter().rev().take(limit / 3) {
            events.push(format!(
                "Comment by {}: \"{}\"",
                comment.agent_name,
                comment.content.chars().take(100).collect::<String>()
            ));
        }

        for action in self.action_log.iter().rev().take(limit / 3) {
            events.push(format!(
                "{} by {} (round {})",
                action.action_type, action.agent_name, action.round_num
            ));
        }

        events.truncate(limit);
        events
    }

    /// Execute an agent action by agent ID and name (avoids borrow issues)
    async fn execute_action_by_id(
        &mut self,
        agent_id: usize,
        agent_name: &str,
        action: AgentAction,
        round: u32,
        _hour: usize,
    ) {
        let platform = if self.simulation.enable_twitter && self.simulation.enable_reddit {
            if round % 2 == 0 { "twitter" } else { "reddit" }
        } else if self.simulation.enable_twitter {
            "twitter"
        } else {
            "reddit"
        };

        let action_record = ActionRecord::from_action(
            &self.simulation.simulation_id,
            round,
            platform,
            agent_id,
            agent_name,
            &action,
            None,
            true,
        );

        match action {
            AgentAction::CreatePost { content, topic } => {
                let post = Post {
                    post_id: format!("post_{}", self.posts.len()),
                    agent_id,
                    agent_name: agent_name.to_string(),
                    platform: platform.to_string(),
                    content,
                    topic,
                    created_at: Utc::now(),
                    likes: 0,
                    dislikes: 0,
                    comments: Vec::new(),
                    shares: 0,
                };
                self.posts.push(post);
            }
            AgentAction::CreateComment { post_id, content } => {
                let comment = Comment {
                    comment_id: format!("comment_{}", self.comments.len()),
                    post_id,
                    agent_id,
                    agent_name: agent_name.to_string(),
                    content,
                    created_at: Utc::now(),
                    likes: 0,
                    dislikes: 0,
                };
                self.comments.push(comment);
            }
            AgentAction::LikePost { post_id } => {
                if let Some(post) = self.posts.iter_mut().find(|p| p.post_id == post_id) {
                    post.likes += 1;
                }
            }
            AgentAction::DislikePost { post_id } => {
                if let Some(post) = self.posts.iter_mut().find(|p| p.post_id == post_id) {
                    post.dislikes += 1;
                }
            }
            AgentAction::SharePost { post_id, .. } => {
                if let Some(post) = self.posts.iter_mut().find(|p| p.post_id == post_id) {
                    post.shares += 1;
                }
            }
            _ => {}
        }

        self.action_log.push(action_record);
        self.total_actions += 1;
    }

    /// Execute an agent action (kept for compatibility)
    #[allow(dead_code)]
    async fn execute_action(
        &mut self,
        agent: &Agent,
        action: AgentAction,
        round: u32,
        hour: usize,
    ) {
        let agent_id = agent.profile.agent_id;
        let agent_name = agent.profile.name.clone();
        self.execute_action_by_id(agent_id, &agent_name, action, round, hour).await;
    }
}
