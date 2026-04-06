//! Agent action types (matching OASIS social media simulation actions)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// All possible agent actions in the simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentAction {
    /// Create a new post with content
    CreatePost {
        content: String,
        topic: Option<String>,
    },
    /// Like a post
    LikePost {
        post_id: String,
    },
    /// Dislike/downvote a post
    DislikePost {
        post_id: String,
    },
    /// Comment on a post
    CreateComment {
        post_id: String,
        content: String,
    },
    /// Like a comment
    LikeComment {
        comment_id: String,
    },
    /// Dislike a comment
    DislikeComment {
        comment_id: String,
    },
    /// Share/retweet a post with optional commentary
    SharePost {
        post_id: String,
        commentary: Option<String>,
    },
    /// Follow another agent
    FollowAgent {
        target_agent_id: usize,
    },
    /// Unfollow another agent
    UnfollowAgent {
        target_agent_id: usize,
    },
    /// Respond to an interview question
    InterviewResponse {
        prompt: String,
        response: String,
    },
    /// No action (passive round)
    None,
}

impl AgentAction {
    pub fn action_type(&self) -> &'static str {
        match self {
            AgentAction::CreatePost { .. } => "create_post",
            AgentAction::LikePost { .. } => "like_post",
            AgentAction::DislikePost { .. } => "dislike_post",
            AgentAction::CreateComment { .. } => "create_comment",
            AgentAction::LikeComment { .. } => "like_comment",
            AgentAction::DislikeComment { .. } => "dislike_comment",
            AgentAction::SharePost { .. } => "share_post",
            AgentAction::FollowAgent { .. } => "follow_agent",
            AgentAction::UnfollowAgent { .. } => "unfollow_agent",
            AgentAction::InterviewResponse { .. } => "interview_response",
            AgentAction::None => "none",
        }
    }

    pub fn to_args_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or_default()
    }
}

/// An executed action record (stored in action log)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRecord {
    pub simulation_id: String,
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

impl ActionRecord {
    pub fn from_action(
        simulation_id: &str,
        round_num: u32,
        platform: &str,
        agent_id: usize,
        agent_name: &str,
        action: &AgentAction,
        result: Option<String>,
        success: bool,
    ) -> Self {
        Self {
            simulation_id: simulation_id.to_string(),
            round_num,
            timestamp: Utc::now(),
            platform: platform.to_string(),
            agent_id,
            agent_name: agent_name.to_string(),
            action_type: action.action_type().to_string(),
            action_args: action.to_args_json(),
            result,
            success,
        }
    }
}

/// Post in the simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub post_id: String,
    pub agent_id: usize,
    pub agent_name: String,
    pub platform: String,
    pub content: String,
    pub topic: Option<String>,
    pub created_at: DateTime<Utc>,
    pub likes: usize,
    pub dislikes: usize,
    pub comments: Vec<Comment>,
    pub shares: usize,
}

/// Comment in the simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub comment_id: String,
    pub post_id: String,
    pub agent_id: usize,
    pub agent_name: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub likes: usize,
    pub dislikes: usize,
}