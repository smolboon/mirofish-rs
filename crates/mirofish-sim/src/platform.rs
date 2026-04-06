//! Platform-specific simulation logic (Twitter vs Reddit)

use std::collections::HashMap;
use chrono::Utc;
use mirofish_core::SimulationPlatform;
use crate::actions::Post;

/// Platform-specific behavior
pub trait Platform: Send + Sync {
    fn platform_type(&self) -> SimulationPlatform;
    fn format_post_content(&self, content: &str) -> String;
    fn format_comment(&self, content: &str) -> String;
    fn max_post_length(&self) -> usize;
    fn supports_threading(&self) -> bool;
    fn calculate_post_visibility(&self, post: &Post, total_posts: usize) -> f64;
}

/// Twitter platform implementation
pub struct TwitterPlatform;

impl Platform for TwitterPlatform {
    fn platform_type(&self) -> SimulationPlatform {
        SimulationPlatform::Twitter
    }

    fn format_post_content(&self, content: &str) -> String {
        if content.len() > 280 {
            format!("{}...", &content[..277])
        } else {
            content.to_string()
        }
    }

    fn format_comment(&self, content: &str) -> String {
        if content.len() > 280 {
            format!("{}...", &content[..277])
        } else {
            content.to_string()
        }
    }

    fn max_post_length(&self) -> usize {
        280
    }

    fn supports_threading(&self) -> bool {
        true
    }

    fn calculate_post_visibility(&self, post: &Post, total_posts: usize) -> f64 {
        let age_hours = (Utc::now() - post.created_at).num_hours() as f64;
        let engagement = (post.likes + post.comments.len()) as f64;
        let recency_factor = (-0.1 * age_hours).exp();
        let engagement_factor = (engagement / (total_posts as f64 + 1.0)).min(1.0);
        0.6 * recency_factor + 0.4 * engagement_factor
    }
}

/// Reddit platform implementation
pub struct RedditPlatform;

impl Platform for RedditPlatform {
    fn platform_type(&self) -> SimulationPlatform {
        SimulationPlatform::Reddit
    }

    fn format_post_content(&self, content: &str) -> String {
        content.to_string()
    }

    fn format_comment(&self, content: &str) -> String {
        content.to_string()
    }

    fn max_post_length(&self) -> usize {
        40000
    }

    fn supports_threading(&self) -> bool {
        true
    }

    fn calculate_post_visibility(&self, post: &Post, total_posts: usize) -> f64 {
        let score = post.likes as f64 - post.dislikes as f64;
        let age_hours = (Utc::now() - post.created_at).num_hours() as f64;
        let score_factor = score / (total_posts as f64 + 1.0);
        let recency_factor = (-0.05 * age_hours).exp();
        (score_factor * 0.7 + recency_factor * 0.3).max(0.0).min(1.0)
    }
}

/// Platform manager
pub struct PlatformManager {
    platforms: HashMap<String, Box<dyn Platform + Send + Sync>>,
}

impl PlatformManager {
    pub fn new() -> Self {
        let mut platforms = HashMap::new();
        platforms.insert("twitter".to_string(), Box::new(TwitterPlatform) as Box<dyn Platform + Send + Sync>);
        platforms.insert("reddit".to_string(), Box::new(RedditPlatform) as Box<dyn Platform + Send + Sync>);
        Self { platforms }
    }

    pub fn get_platform(&self, name: &str) -> Option<&(dyn Platform + Send + Sync)> {
        self.platforms.get(name).map(|p| p.as_ref())
    }

    pub fn format_post_content(&self, content: &str, platform: &str) -> String {
        match self.get_platform(platform) {
            Some(p) => p.format_post_content(content),
            None => content.to_string(),
        }
    }

    pub fn format_comment(&self, content: &str, platform: &str) -> String {
        match self.get_platform(platform) {
            Some(p) => p.format_comment(content),
            None => content.to_string(),
        }
    }
}