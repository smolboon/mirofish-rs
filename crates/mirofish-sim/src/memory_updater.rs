//! Zep Graph Memory Updater
//!
//! Dynamically updates the Zep knowledge graph with agent activities
//! during simulation. Batches activities by platform and sends them
//! as text descriptions for entity/relationship extraction.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::Mutex;
use tracing::{debug, info, warn, error, Instrument};

use mirofish_graph::client::ZepClient;

/// An agent activity record for graph memory update
#[derive(Debug, Clone)]
pub struct AgentActivity {
    pub platform: String,       // "twitter" or "reddit"
    pub agent_id: usize,
    pub agent_name: String,
    pub action_type: String,    // "CREATE_POST", "LIKE_POST", etc.
    pub action_args: serde_json::Value,
    pub round_num: u32,
    pub timestamp: String,
}

impl AgentActivity {
    /// Convert activity to natural language description for Zep
    pub fn to_episode_text(&self) -> String {
        let description = match self.action_type.as_str() {
            "CREATE_POST" => self._describe_create_post(),
            "LIKE_POST" => self._describe_like_post(),
            "DISLIKE_POST" => self._describe_dislike_post(),
            "REPOST" => self._describe_repost(),
            "QUOTE_POST" => self._describe_quote_post(),
            "FOLLOW" => self._describe_follow(),
            "CREATE_COMMENT" => self._describe_create_comment(),
            "LIKE_COMMENT" => self._describe_like_comment(),
            "DISLIKE_COMMENT" => self._describe_dislike_comment(),
            "SEARCH_POSTS" => self._describe_search(),
            "SEARCH_USER" => self._describe_search_user(),
            "MUTE" => self._describe_mute(),
            _ => self._describe_generic(),
        };

        format!("{}: {}", self.agent_name, description)
    }

    fn _describe_create_post(&self) -> String {
        let content = self.action_args.get("content").and_then(|v| v.as_str()).unwrap_or("");
        if !content.is_empty() {
            return format!("发布了一条帖子：「{}」", content);
        }
        "发布了一条帖子".to_string()
    }

    fn _describe_like_post(&self) -> String {
        let post_content = self.action_args.get("post_content").and_then(|v| v.as_str()).unwrap_or("");
        let post_author = self.action_args.get("post_author_name").and_then(|v| v.as_str()).unwrap_or("");
        if !post_content.is_empty() && !post_author.is_empty() {
            format!("点赞了{}的帖子：「{}」", post_author, post_content)
        } else if !post_content.is_empty() {
            format!("点赞了一条帖子：「{}」", post_content)
        } else if !post_author.is_empty() {
            format!("点赞了{}的一条帖子", post_author)
        } else {
            "点赞了一条帖子".to_string()
        }
    }

    fn _describe_dislike_post(&self) -> String {
        let post_content = self.action_args.get("post_content").and_then(|v| v.as_str()).unwrap_or("");
        let post_author = self.action_args.get("post_author_name").and_then(|v| v.as_str()).unwrap_or("");
        if !post_content.is_empty() && !post_author.is_empty() {
            format!("踩了{}的帖子：「{}」", post_author, post_content)
        } else if !post_content.is_empty() {
            format!("踩了一条帖子：「{}」", post_content)
        } else if !post_author.is_empty() {
            format!("踩了{}的一条帖子", post_author)
        } else {
            "踩了一条帖子".to_string()
        }
    }

    fn _describe_repost(&self) -> String {
        let original_content = self.action_args.get("original_content").and_then(|v| v.as_str()).unwrap_or("");
        let original_author = self.action_args.get("original_author_name").and_then(|v| v.as_str()).unwrap_or("");
        if !original_content.is_empty() && !original_author.is_empty() {
            format!("转发了{}的帖子：「{}」", original_author, original_content)
        } else if !original_content.is_empty() {
            format!("转发了一条帖子：「{}」", original_content)
        } else if !original_author.is_empty() {
            format!("转发了{}的一条帖子", original_author)
        } else {
            "转发了一条帖子".to_string()
        }
    }

    fn _describe_quote_post(&self) -> String {
        let original_content = self.action_args.get("original_content").and_then(|v| v.as_str()).unwrap_or("");
        let original_author = self.action_args.get("original_author_name").and_then(|v| v.as_str()).unwrap_or("");
        let quote_content = self.action_args.get("quote_content")
            .or_else(|| self.action_args.get("content"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let mut base = if !original_content.is_empty() && !original_author.is_empty() {
            format!("引用了{}的帖子「{}」", original_author, original_content)
        } else if !original_content.is_empty() {
            format!("引用了一条帖子「{}」", original_content)
        } else if !original_author.is_empty() {
            format!("引用了{}的一条帖子", original_author)
        } else {
            "引用了一条帖子".to_string()
        };
        if !quote_content.is_empty() {
            base.push_str(&format!("，并评论道：「{}」", quote_content));
        }
        base
    }

    fn _describe_follow(&self) -> String {
        let target = self.action_args.get("target_user_name").and_then(|v| v.as_str()).unwrap_or("");
        if !target.is_empty() {
            return format!("关注了用户「{}」", target);
        }
        "关注了一个用户".to_string()
    }

    fn _describe_create_comment(&self) -> String {
        let content = self.action_args.get("content").and_then(|v| v.as_str()).unwrap_or("");
        let post_content = self.action_args.get("post_content").and_then(|v| v.as_str()).unwrap_or("");
        let post_author = self.action_args.get("post_author_name").and_then(|v| v.as_str()).unwrap_or("");
        if !content.is_empty() {
            if !post_content.is_empty() && !post_author.is_empty() {
                format!("在{}的帖子「{}」下评论道：「{}」", post_author, post_content, content)
            } else if !post_content.is_empty() {
                format!("在帖子「{}」下评论道：「{}」", post_content, content)
            } else if !post_author.is_empty() {
                format!("在{}的帖子下评论道：「{}」", post_author, content)
            } else {
                format!("评论道：「{}」", content)
            }
        } else {
            "发表了评论".to_string()
        }
    }

    fn _describe_like_comment(&self) -> String {
        let comment_content = self.action_args.get("comment_content").and_then(|v| v.as_str()).unwrap_or("");
        let comment_author = self.action_args.get("comment_author_name").and_then(|v| v.as_str()).unwrap_or("");
        if !comment_content.is_empty() && !comment_author.is_empty() {
            format!("点赞了{}的评论：「{}」", comment_author, comment_content)
        } else if !comment_content.is_empty() {
            format!("点赞了一条评论：「{}」", comment_content)
        } else if !comment_author.is_empty() {
            format!("点赞了{}的一条评论", comment_author)
        } else {
            "点赞了一条评论".to_string()
        }
    }

    fn _describe_dislike_comment(&self) -> String {
        let comment_content = self.action_args.get("comment_content").and_then(|v| v.as_str()).unwrap_or("");
        let comment_author = self.action_args.get("comment_author_name").and_then(|v| v.as_str()).unwrap_or("");
        if !comment_content.is_empty() && !comment_author.is_empty() {
            format!("踩了{}的评论：「{}」", comment_author, comment_content)
        } else if !comment_content.is_empty() {
            format!("踩了一条评论：「{}」", comment_content)
        } else if !comment_author.is_empty() {
            format!("踩了{}的一条评论", comment_author)
        } else {
            "踩了一条评论".to_string()
        }
    }

    fn _describe_search(&self) -> String {
        let query = self.action_args.get("query")
            .or_else(|| self.action_args.get("keyword"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if !query.is_empty() {
            format!("搜索了「{}」", query)
        } else {
            "进行了搜索".to_string()
        }
    }

    fn _describe_search_user(&self) -> String {
        let query = self.action_args.get("query")
            .or_else(|| self.action_args.get("username"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if !query.is_empty() {
            format!("搜索了用户「{}」", query)
        } else {
            "搜索了用户".to_string()
        }
    }

    fn _describe_mute(&self) -> String {
        let target = self.action_args.get("target_user_name").and_then(|v| v.as_str()).unwrap_or("");
        if !target.is_empty() {
            return format!("屏蔽了用户「{}」", target);
        }
        "屏蔽了一个用户".to_string()
    }

    fn _describe_generic(&self) -> String {
        format!("执行了{}操作", self.action_type)
    }
}

/// Statistics for the memory updater
#[derive(Debug, Clone, Default)]
pub struct MemoryUpdaterStats {
    pub total_activities: usize,
    pub batches_sent: usize,
    pub items_sent: usize,
    pub failed_batches: usize,
    pub skipped_count: usize,
}

/// Configuration for the graph memory updater
pub struct MemoryUpdaterConfig {
    pub batch_size: usize,
    pub send_interval_ms: u64,
    pub max_retries: usize,
    pub retry_delay_ms: u64,
}

impl Default for MemoryUpdaterConfig {
    fn default() -> Self {
        Self {
            batch_size: 5,
            send_interval_ms: 500,
            max_retries: 3,
            retry_delay_ms: 2000,
        }
    }
}

/// Graph Memory Updater - batches and sends agent activities to Zep
pub struct GraphMemoryUpdater {
    graph_id: String,
    zep: Arc<ZepClient>,
    config: MemoryUpdaterConfig,
    /// Per-platform activity buffers
    state: Arc<UpdaterState>,
    /// Channel sender for external use
    pub tx: tokio::sync::mpsc::UnboundedSender<AgentActivity>,
    /// Running flag
    running: Arc<AtomicBool>,
}

struct UpdaterStateInner {
    platform_buffers: std::sync::Mutex<HashMap<String, Vec<AgentActivity>>>,
    stats: std::sync::Mutex<MemoryUpdaterStats>,
}

type UpdaterState = Arc<UpdaterStateInner>;

/// Send a batch of activities to Zep with retry
async fn send_batch(
    zep: &ZepClient,
    graph_id: &str,
    batch: &[AgentActivity],
    config: &MemoryUpdaterConfig,
    stats: &std::sync::Mutex<MemoryUpdaterStats>,
) {
    if batch.is_empty() {
        return;
    }

    // Combine all activities into one text
    let combined_text: String = batch.iter()
        .map(|a| a.to_episode_text())
        .collect::<Vec<_>>()
        .join("\n");

    // Retry logic
    for attempt in 0..config.max_retries {
        match zep.add_document(graph_id, &combined_text).await {
            Ok(_) => {
                stats.lock().unwrap().batches_sent += 1;
                stats.lock().unwrap().items_sent += batch.len();
                debug!(
                    "Successfully sent {} activities to graph {}",
                    batch.len(), graph_id
                );
                return;
            }
            Err(e) if attempt < config.max_retries - 1 => {
                warn!(
                    "Failed to send batch (attempt {}/{}): {}",
                    attempt + 1, config.max_retries, e
                );
                tokio::time::sleep(
                    tokio::time::Duration::from_millis(config.retry_delay_ms * (attempt as u64 + 1))
                ).await;
            }
            Err(e) => {
                error!(
                    "Failed to send batch after {} retries: {}",
                    config.max_retries, e
                );
                stats.lock().unwrap().failed_batches += 1;
                return;
            }
        }
    }
}

/// Flush all remaining activities in buffers
async fn flush_remaining(
    zep: &ZepClient,
    graph_id: &str,
    state: &UpdaterState,
    config: &MemoryUpdaterConfig,
) {
    let mut buffers = state.platform_buffers.lock().unwrap();
    for (platform, activities) in buffers.iter_mut() {
        if !activities.is_empty() {
            info!(
                "Flushing {} remaining activities for platform {}",
                activities.len(),
                platform
            );
            let batch: Vec<AgentActivity> = std::mem::take(activities);
            send_batch(zep, graph_id, &batch, config, &state.stats).await;
        }
    }
    buffers.clear();
}

impl GraphMemoryUpdater {
    /// Create a new graph memory updater
    pub fn new(
        graph_id: String,
        zep: Arc<ZepClient>,
        config: MemoryUpdaterConfig,
    ) -> Self {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let state = Arc::new(UpdaterStateInner {
            platform_buffers: std::sync::Mutex::new(HashMap::new()),
            stats: std::sync::Mutex::new(MemoryUpdaterStats::default()),
        });

        Self {
            graph_id,
            zep,
            config,
            state,
            tx,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Start the background update loop (spawns a tokio task)
    pub fn start(&self) {
        self.running.store(true, Ordering::SeqCst);
        info!("GraphMemoryUpdater started: graph_id={}", self.graph_id);

        let running = self.running.clone();
        let zep = self.zep.clone();
        let state = self.state.clone();
        let config = self.config.clone();
        let graph_id = self.graph_id.clone();
        let send_interval = tokio::time::Duration::from_millis(self.config.send_interval_ms);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(send_interval);
            let mut rx = self.tx.subscribe();

            while running.load(Ordering::SeqCst) {
                tokio::select! {
                    _ = interval.tick() => {
                        // Check buffers and send if full
                        let mut buffers = state.platform_buffers.lock().unwrap();
                        for (_platform, activities) in buffers.iter_mut() {
                            if activities.len() >= config.batch_size {
                                let batch: Vec<AgentActivity> = activities.drain(..config.batch_size).collect();
                                drop(buffers);
                                send_batch(&zep, &graph_id, &batch, &config, &state.stats).await;
                                buffers = state.platform_buffers.lock().unwrap();
                            }
                        }
                    }
                    Ok(activity) = rx.recv() => {
                        // Skip DO_NOTHING
                        if activity.action_type == "DO_NOTHING" {
                            state.stats.lock().unwrap().skipped_count += 1;
                            continue;
                        }
                        let platform = activity.platform.to_lowercase();
                        let mut buffers = state.platform_buffers.lock().unwrap();
                        buffers.entry(platform).or_default().push(activity);
                    }
                }
            }

            // Flush remaining
            flush_remaining(&zep, &graph_id, &state, &config).await;
            info!("GraphMemoryUpdater background task finished");
        }.instrument(tracing::info_span!("graph_memory_updater", graph_id = %graph_id)));
    }

    /// Stop the background update loop
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    /// Get statistics
    pub fn get_stats(&self) -> MemoryUpdaterStats {
        self.state.stats.lock().unwrap().clone()
    }

    /// Add an activity to the queue
    pub fn add_activity(&self, activity: AgentActivity) -> Result<(), String> {
        if activity.action_type == "DO_NOTHING" {
            self.state.stats.lock().unwrap().skipped_count += 1;
            return Ok(());
        }
        self.tx.send(activity)
            .map_err(|e| format!("Failed to send activity: {}", e))
    }

    /// Add activity from JSON dict (from actions.jsonl)
    pub fn add_activity_from_dict(&self, data: serde_json::Value, platform: &str) -> Result<(), String> {
        // Skip event-type entries
        if data.get("event_type").is_some() {
            return Ok(());
        }
        // Skip entries without agent_id
        if data.get("agent_id").is_none() {
            return Ok(());
        }

        let activity = AgentActivity {
            platform: platform.to_string(),
            agent_id: data.get("agent_id").and_then(|v| v.as_u64()).unwrap_or(0) as usize,
            agent_name: data.get("agent_name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            action_type: data.get("action_type").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            action_args: data.get("action_args").cloned().unwrap_or(serde_json::Value::Object(Default::default())),
            round_num: data.get("round").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            timestamp: data.get("timestamp").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        };
        self.add_activity(activity)
    }
}

/// Manager for multiple graph memory updaters (one per simulation)
pub struct GraphMemoryManager {
    updaters: Arc<std::sync::Mutex<HashMap<String, GraphMemoryUpdater>>>,
}

impl GraphMemoryManager {
    pub fn new() -> Self {
        Self {
            updaters: Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }

    /// Create and start an updater for a simulation
    pub fn create_updater(
        &self,
        simulation_id: &str,
        graph_id: &str,
        zep: Arc<ZepClient>,
    ) {
        let mut updaters = self.updaters.lock().unwrap();

        // Stop existing updater if present
        if let Some(existing) = updaters.remove(simulation_id) {
            existing.stop();
        }

        let updater = GraphMemoryUpdater::new(
            graph_id.to_string(),
            zep,
            MemoryUpdaterConfig::default(),
        );

        info!(
            "Created graph memory updater: simulation_id={}, graph_id={}",
            simulation_id, graph_id
        );

        updaters.insert(simulation_id.to_string(), updater);
    }

    /// Get an updater by simulation ID
    pub fn get_updater(&self, simulation_id: &str) -> Option<&GraphMemoryUpdater> {
        self.updaters.lock().unwrap().get(simulation_id)
    }

    /// Stop and remove an updater
    pub fn stop_updater(&self, simulation_id: &str) {
        let mut updaters = self.updaters.lock().unwrap();
        if let Some(updater) = updaters.remove(simulation_id) {
            updater.stop();
            info!("Stopped graph memory updater: simulation_id={}", simulation_id);
        }
    }

    /// Stop all updaters
    pub fn stop_all(&self) {
        let mut updaters = self.updaters.lock().unwrap();
        for (sim_id, updater) in updaters.drain() {
            updater.stop();
            info!("Stopped graph memory updater: simulation_id={}", sim_id);
        }
    }

    /// Get stats for all updaters
    pub fn get_all_stats(&self) -> HashMap<String, MemoryUpdaterStats> {
        let updaters = self.updaters.lock().unwrap();
        updaters.iter()
            .map(|(id, updater)| (id.clone(), updater.get_stats()))
            .collect()
    }
}