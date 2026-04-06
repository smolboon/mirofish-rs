//! Agent behavior and decision logic

use std::collections::HashMap;
use rand::thread_rng;
use rand::RngCore;

use mirofish_core::{AgentConfig, AgentProfile};
use mirofish_llm::LLMClient;
use crate::actions::{Post, Comment, AgentAction};

/// An active agent in the simulation
pub struct Agent {
    pub profile: AgentProfile,
    pub config: AgentConfig,
    pub memory: AgentMemory,
    pub is_active: bool,
}

impl Agent {
    pub fn new(profile: AgentProfile, config: AgentConfig) -> Self {
        Self {
            profile,
            config,
            memory: AgentMemory::new(),
            is_active: true,
        }
    }

    /// Determine if this agent should be active in the current round
    pub fn should_act(&self, _round: u32, hour: usize) -> bool {
        if !self.is_active {
            return false;
        }

        let is_peak = self.profile.activity_pattern.peak_hours.contains(&hour);
        let base_chance = self.config.activity_level;
        let peak_bonus = if is_peak { 1.5 } else { 0.7 };

        let mut rng = thread_rng();
        let rand_val: f64 = (rng.next_u64() as f64) / (u64::MAX as f64);
        rand_val < (base_chance * peak_bonus)
    }

    /// Decide the next action for this agent
    pub async fn decide_next_action(
        &self,
        llm: &LLMClient,
        available_posts: &[Post],
        available_comments: &[Comment],
        recent_events: &[String],
        round: u32,
    ) -> Result<AgentAction, String> {
        let action_choice = self.select_action_type(available_posts, available_comments);

        match action_choice {
            ActionType::CreatePost => {
                self.generate_post(llm, recent_events, round).await
            }
            ActionType::Comment => {
                self.generate_comment(llm, available_posts, recent_events, round).await
            }
            ActionType::Like => {
                self.select_like_action(available_posts)
            }
            ActionType::Dislike => {
                self.select_dislike_action(available_posts)
            }
            ActionType::Share => {
                self.generate_share(llm, available_posts, recent_events, round).await
            }
            ActionType::None => Ok(AgentAction::None),
        }
    }

    fn select_action_type(
        &self,
        available_posts: &[Post],
        _available_comments: &[Comment],
    ) -> ActionType {
        let mut rng = thread_rng();
        let roll: f64 = (rng.next_u64() as f64) / (u64::MAX as f64);

        if available_posts.is_empty() {
            return ActionType::None;
        }

        if roll < self.config.posting_probability {
            let reaction_roll: f64 = (rng.next_u64() as f64) / (u64::MAX as f64);
            if reaction_roll < 0.3 {
                ActionType::CreatePost
            } else if reaction_roll < 0.7 {
                ActionType::Comment
            } else if reaction_roll < 0.85 {
                ActionType::Like
            } else if reaction_roll < 0.95 {
                ActionType::Share
            } else {
                ActionType::Dislike
            }
        } else if roll < self.config.comment_probability {
            ActionType::Comment
        } else if roll < self.config.like_probability {
            ActionType::Like
        } else {
            ActionType::None
        }
    }

    fn pick_random_element<'a>(items: &'a [Post]) -> Option<&'a Post> {
        if items.is_empty() {
            return None;
        }
        let mut rng = thread_rng();
        let idx: usize = (rng.next_u64() as usize) % items.len();
        Some(&items[idx])
    }

    async fn generate_post(
        &self,
        llm: &LLMClient,
        recent_events: &[String],
        round: u32,
    ) -> Result<AgentAction, String> {
        let system_prompt = format!(
            "You are {}. Your interests: {}. Your stance on the topic: {}. Communication style: {}",
            self.profile.name,
            self.profile.persona.interests.join(", "),
            self.profile.persona.stance_on_topic,
            self.profile.persona.communication_style,
        );

        let events_context = if recent_events.is_empty() {
            "No recent events.".to_string()
        } else {
            format!(
                "Recent events:\n{}",
                recent_events
                    .iter()
                    .take(5)
                    .map(|e| format!("- {}", e))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        };

        let user_prompt = format!(
            "Simulation round {}. {}\n\nWrite a post in character. Under 280 chars.",
            round, events_context
        );

        let content = llm.chat(&system_prompt, &user_prompt).await
            .map_err(|e| format!("LLM error: {}", e))?;

        Ok(AgentAction::CreatePost {
            content,
            topic: self.profile.persona.interests.first().cloned(),
        })
    }

    async fn generate_comment(
        &self,
        llm: &LLMClient,
        available_posts: &[Post],
        _recent_events: &[String],
        _round: u32,
    ) -> Result<AgentAction, String> {
        let target_post = match Self::pick_random_element(available_posts) {
            Some(p) => p,
            None => return Ok(AgentAction::None),
        };

        let system_prompt = format!(
            "You are {}. Stance: {}. Style: {}.",
            self.profile.name,
            self.profile.persona.stance_on_topic,
            self.profile.persona.communication_style,
        );

        let user_prompt = format!(
            "Post from {}: {}\n\nWrite a comment (under 200 chars).",
            target_post.agent_name, target_post.content
        );

        let content = llm.chat(&system_prompt, &user_prompt).await
            .map_err(|e| format!("LLM error: {}", e))?;

        Ok(AgentAction::CreateComment {
            post_id: target_post.post_id.clone(),
            content,
        })
    }

    fn select_like_action(&self, available_posts: &[Post]) -> Result<AgentAction, String> {
        let target_post = match Self::pick_random_element(available_posts) {
            Some(p) => p,
            None => return Ok(AgentAction::None),
        };
        Ok(AgentAction::LikePost {
            post_id: target_post.post_id.clone(),
        })
    }

    fn select_dislike_action(&self, available_posts: &[Post]) -> Result<AgentAction, String> {
        let target_post = match Self::pick_random_element(available_posts) {
            Some(p) => p,
            None => return Ok(AgentAction::None),
        };
        Ok(AgentAction::DislikePost {
            post_id: target_post.post_id.clone(),
        })
    }

    async fn generate_share(
        &self,
        llm: &LLMClient,
        available_posts: &[Post],
        _recent_events: &[String],
        _round: u32,
    ) -> Result<AgentAction, String> {
        let target_post = match Self::pick_random_element(available_posts) {
            Some(p) => p,
            None => return Ok(AgentAction::None),
        };

        let system_prompt = format!(
            "You are {}. Stance: {}. Write SHORT commentary (under 200 chars).",
            self.profile.name, self.profile.persona.stance_on_topic,
        );

        let commentary = llm
            .chat(
                &system_prompt,
                &format!(
                    "Post: \"{}\"\n\nAdd commentary:",
                    target_post.content
                ),
            )
            .await
            .map_err(|e| format!("LLM error: {}", e))?;

        Ok(AgentAction::SharePost {
            post_id: target_post.post_id.clone(),
            commentary: Some(commentary),
        })
    }
}

/// Agent's memory
pub struct AgentMemory {
    pub past_posts: Vec<String>,
    pub past_comments: Vec<String>,
    pub interactions: HashMap<u64, usize>,
    pub recent_memories: Vec<String>,
}

impl AgentMemory {
    pub fn new() -> Self {
        Self {
            past_posts: Vec::new(),
            past_comments: Vec::new(),
            interactions: HashMap::new(),
            recent_memories: Vec::new(),
        }
    }

    pub fn add_memory(&mut self, memory: String) {
        self.recent_memories.push(memory);
        if self.recent_memories.len() > 50 {
            self.recent_memories.drain(..self.recent_memories.len() - 50);
        }
    }

    pub fn get_recent_context(&self, limit: usize) -> String {
        self.recent_memories
            .iter()
            .rev()
            .take(limit)
            .map(|m| format!("- {}", m))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Action type selection
#[derive(Debug, Clone, PartialEq)]
enum ActionType {
    CreatePost,
    Comment,
    Like,
    Dislike,
    Share,
    None,
}