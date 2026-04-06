//! Task manager - in-memory task tracking with watch channels for SSE

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use chrono::Utc;
use tokio::sync::watch;
use tracing::debug;

use mirofish_core::{Task, TaskStatus, TaskProgressUpdate, TaskStatusResponse, TaskDictResponse};

/// In-memory task store
#[derive(Clone)]
pub struct TaskManager {
    tasks: Arc<RwLock<HashMap<String, Task>>>,
    /// Watch senders for each task (for SSE streaming)
    watchers: Arc<RwLock<HashMap<String, watch::Sender<Task>>>>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            watchers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new task
    pub fn create_task(&self, task_type: &str, metadata: serde_json::Value) -> String {
        let task = Task::new(task_type, metadata);
        let task_id = task.task_id.clone();

        // Create a watch channel for this task
        let (tx, _rx) = watch::channel(task.clone());

        {
            let mut tasks = self.tasks.write().unwrap();
            tasks.insert(task_id.clone(), task);
        }
        {
            let mut watchers = self.watchers.write().unwrap();
            watchers.insert(task_id.clone(), tx);
        }

        debug!("Created task: {}", task_id);
        task_id
    }

    /// Get task by ID
    pub fn get_task(&self, task_id: &str) -> Option<Task> {
        self.tasks.read().unwrap().get(task_id).cloned()
    }

    /// Update task status
    pub fn update_task(
        &self,
        task_id: &str,
        status: Option<TaskStatus>,
        progress: Option<u8>,
        message: Option<&str>,
        progress_detail: Option<serde_json::Value>,
    ) -> Option<Task> {
        let mut tasks = self.tasks.write().unwrap();
        let task = tasks.get_mut(task_id)?;

        if let Some(s) = status {
            task.status = s;
        }
        if let Some(p) = progress {
            task.progress = p;
        }
        if let Some(m) = message {
            task.message = m.to_string();
        }
        if let Some(d) = progress_detail {
            task.progress_detail = Some(d);
        }
        task.updated_at = Utc::now();

        // Notify watchers
        if let Some(tx) = self.watchers.read().unwrap().get(task_id) {
            let _ = tx.send(task.clone());
        }

        Some(task.clone())
    }

    /// Mark task as completed with result
    pub fn complete_task(&self, task_id: &str, result: serde_json::Value) -> Option<Task> {
        self.update_task(
            task_id,
            Some(TaskStatus::Completed),
            Some(100),
            Some("Task completed"),
            None,
        );

        let mut tasks = self.tasks.write().unwrap();
        if let Some(task) = tasks.get_mut(task_id) {
            task.result = Some(result);
            Some(task.clone())
        } else {
            None
        }
    }

    /// Mark task as failed
    pub fn fail_task(&self, task_id: &str, error: &str) -> Option<Task> {
        let task = self.update_task(
            task_id,
            Some(TaskStatus::Failed),
            None,
            Some(&format!("Task failed: {}", error)),
            None,
        );

        let mut tasks = self.tasks.write().unwrap();
        if let Some(t) = tasks.get_mut(task_id) {
            t.error = Some(error.to_string());
            Some(t.clone())
        } else {
            task
        }
    }

    /// List all tasks
    pub fn list_tasks(&self) -> Vec<Task> {
        self.tasks.read().unwrap().values().cloned().collect()
    }

    /// Get a watch receiver for a task (for SSE)
    pub fn watch_task(&self, task_id: &str) -> Option<watch::Receiver<Task>> {
        self.watchers.read().unwrap().get(task_id).map(|tx| tx.subscribe())
    }

    /// Clean up old completed tasks (older than given duration)
    pub fn cleanup_old_tasks(&self, max_age: Duration) {
        let cutoff = Utc::now() - chrono::Duration::seconds(max_age.as_secs() as i64);
        let mut tasks = self.tasks.write().unwrap();
        tasks.retain(|_, task| {
            task.updated_at > cutoff || task.status != TaskStatus::Completed
        });
    }
}