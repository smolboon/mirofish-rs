//! Interview mode - post-simulation agent interaction

use std::collections::HashMap;
use tracing::debug;

use mirofish_core::{AgentProfile, ChatMessage};
use mirofish_llm::LLMClient;
use crate::actions::{Post, Comment, ActionRecord};

/// Interview session with a specific agent
pub struct InterviewSession {
    pub agent_id: usize,
    pub agent_profile: AgentProfile,
    pub chat_history: Vec<ChatMessage>,
    pub agent_actions: Vec<ActionRecord>,
    pub agent_posts: Vec<Post>,
    pub agent_comments: Vec<Comment>,
}

impl InterviewSession {
    pub fn new(
        agent_id: usize,
        agent_profile: AgentProfile,
        actions: Vec<ActionRecord>,
        posts: Vec<Post>,
        comments: Vec<Comment>,
    ) -> Self {
        Self {
            agent_id,
            agent_profile,
            chat_history: Vec::new(),
            agent_actions: actions,
            agent_posts: posts,
            agent_comments: comments,
        }
    }

    /// Get agent's activity summary
    pub fn activity_summary(&self) -> String {
        let total_posts = self.agent_posts.len();
        let total_comments = self.agent_comments.len();
        let total_actions = self.agent_actions.len();

        format!(
            "Agent {} (@{}) performed {} actions: {} posts, {} comments.",
            self.agent_profile.name,
            self.agent_profile.username,
            total_actions,
            total_posts,
            total_comments,
        )
    }

    /// Get agent's most engaged posts
    pub fn top_posts(&self, limit: usize) -> Vec<&Post> {
        let mut posts: Vec<&Post> = self.agent_posts.iter().collect();
        posts.sort_by(|a, b| {
            let a_score = a.likes + a.comments.len();
            let b_score = b.likes + b.comments.len();
            b_score.cmp(&a_score)
        });
        posts.into_iter().take(limit).collect()
    }
}

/// Interview manager for multiple agents
pub struct InterviewManager {
    pub sessions: HashMap<usize, InterviewSession>,
}

impl InterviewManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    /// Create an interview session for an agent
    pub fn create_session(
        &mut self,
        agent_id: usize,
        profile: AgentProfile,
        actions: Vec<ActionRecord>,
        posts: Vec<Post>,
        comments: Vec<Comment>,
    ) {
        let session = InterviewSession::new(agent_id, profile, actions, posts, comments);
        self.sessions.insert(agent_id, session);
    }

    /// Get a session
    pub fn get_session(&self, agent_id: usize) -> Option<&InterviewSession> {
        self.sessions.get(&agent_id)
    }

    /// Get a mutable session
    pub fn get_session_mut(&mut self, agent_id: usize) -> Option<&mut InterviewSession> {
        self.sessions.get_mut(&agent_id)
    }

    /// List all available agents for interview
    pub fn list_agents(&self) -> Vec<(usize, &str, &str)> {
        self.sessions
            .iter()
            .map(|(id, session)| {
                (
                    *id,
                    session.agent_profile.name.as_str(),
                    session.agent_profile.username.as_str(),
                )
            })
            .collect()
    }
}

/// Generate interview response using LLM
pub async fn generate_interview_response(
    llm: &LLMClient,
    session: &InterviewSession,
    user_message: &str,
) -> Result<String, String> {
    debug!(
        "Generating interview response for agent {} (id: {})",
        session.agent_profile.name, session.agent_id
    );

    // Build system prompt with agent's persona and simulation history
    let system_prompt = format!(
        "You are roleplaying as {}. \n\
        Username: @{}\n\
        Bio: {}\n\
        Personality: {:?}\n\
        Interests: {}\n\
        Communication style: {}\n\
        Stance on topic: {}\n\
        Demographics: {}, {}, {}\n\n\
        You participated in a social media simulation. \n\
        Your activity summary: {}\n\n\
        Stay in character at all times. Respond naturally as this person would, \
        drawing on your simulated experiences and actions.",
        session.agent_profile.name,
        session.agent_profile.username,
        session.agent_profile.bio,
        session.agent_profile.persona.personality_traits,
        session.agent_profile.persona.interests.join(", "),
        session.agent_profile.persona.communication_style,
        session.agent_profile.persona.stance_on_topic,
        session.agent_profile.demographics.occupation,
        session.agent_profile.demographics.location,
        session.agent_profile.demographics.age_group,
        session.activity_summary(),
    );

    // Build chat history context
    let mut chat_history = String::new();
    for msg in session.chat_history.iter().rev().take(10).rev() {
        chat_history.push_str(&format!("{}: {}\n", msg.role, msg.content));
    }

    // Add agent's recent actions as context
    let recent_actions = session
        .agent_actions
        .iter()
        .rev()
        .take(5)
        .map(|a| format!("- {} (round {})", a.action_type, a.round_num))
        .collect::<Vec<_>>()
        .join("\n");

    let user_prompt = if chat_history.is_empty() {
        format!("User asks: {}\n\nRecent actions:\n{}", user_message, recent_actions)
    } else {
        format!(
            "Chat history:\n{}\n\nUser asks: {}\n\nRecent actions:\n{}",
            chat_history, user_message, recent_actions
        )
    };

    llm.chat(&system_prompt, &user_prompt).await
        .map_err(|e| format!("LLM error: {}", e))
}