//! API client for the MiroFish backend

use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::RequestInit;

/// API client for communicating with the backend
#[derive(Clone)]
pub struct ApiClient {
    base_url: String,
}

impl Default for ApiClient {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8080".to_string(),
        }
    }
}

impl ApiClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
        }
    }

    /// Create a new project
    pub async fn create_project(
        &self,
        name: &str,
        simulation_requirement: &str,
    ) -> Result<serde_json::Value, String> {
        self.post(
            "/api/project/create",
            serde_json::json!({
                "name": name,
                "simulation_requirement": simulation_requirement,
            }),
        )
        .await
    }

    /// Generate ontology
    pub async fn generate_ontology(
        &self,
        project_id: &str,
        text: &str,
        simulation_requirement: &str,
    ) -> Result<serde_json::Value, String> {
        self.post(
            "/api/graph/ontology",
            serde_json::json!({
                "project_id": project_id,
                "text": text,
                "simulation_requirement": simulation_requirement,
            }),
        )
        .await
    }

    /// Build graph
    pub async fn build_graph(
        &self,
        project_id: &str,
        graph_id: &str,
        text: &str,
        ontology: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        self.post(
            "/api/graph/build",
            serde_json::json!({
                "project_id": project_id,
                "graph_id": graph_id,
                "text": text,
                "ontology": ontology,
            }),
        )
        .await
    }

    /// Get task status
    pub async fn get_task_status(&self, task_id: &str) -> Result<serde_json::Value, String> {
        self.get(&format!("/api/graph/task/{}", task_id)).await
    }

    /// Create simulation
    pub async fn create_simulation(
        &self,
        project_id: &str,
        graph_id: &str,
        enable_twitter: bool,
        enable_reddit: bool,
    ) -> Result<serde_json::Value, String> {
        self.post(
            "/api/simulation/create",
            serde_json::json!({
                "project_id": project_id,
                "graph_id": graph_id,
                "enable_twitter": enable_twitter,
                "enable_reddit": enable_reddit,
            }),
        )
        .await
    }

    /// Prepare simulation
    pub async fn prepare_simulation(
        &self,
        simulation_id: &str,
        graph_id: &str,
        simulation_requirement: &str,
        document_text: &str,
        enable_twitter: bool,
        enable_reddit: bool,
    ) -> Result<serde_json::Value, String> {
        self.post(
            "/api/simulation/prepare",
            serde_json::json!({
                "simulation_id": simulation_id,
                "graph_id": graph_id,
                "simulation_requirement": simulation_requirement,
                "document_text": document_text,
                "enable_twitter": enable_twitter,
                "enable_reddit": enable_reddit,
            }),
        )
        .await
    }

    /// Start simulation
    pub async fn start_simulation(
        &self,
        simulation_id: &str,
        project_id: &str,
        graph_id: &str,
        enable_twitter: bool,
        enable_reddit: bool,
        simulation_config: &serde_json::Value,
        profiles: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        self.post(
            "/api/simulation/start",
            serde_json::json!({
                "simulation_id": simulation_id,
                "project_id": project_id,
                "graph_id": graph_id,
                "enable_twitter": enable_twitter,
                "enable_reddit": enable_reddit,
                "simulation_config": simulation_config,
                "profiles": profiles,
            }),
        )
        .await
    }

    /// Get simulation status
    pub async fn get_simulation_status(&self, sim_id: &str) -> Result<serde_json::Value, String> {
        self.get(&format!("/api/simulation/status/{}", sim_id)).await
    }

    /// Generate report
    pub async fn generate_report(
        &self,
        simulation_id: &str,
        graph_id: &str,
        simulation_requirement: &str,
    ) -> Result<serde_json::Value, String> {
        self.post(
            "/api/report/generate",
            serde_json::json!({
                "simulation_id": simulation_id,
                "graph_id": graph_id,
                "simulation_requirement": simulation_requirement,
            }),
        )
        .await
    }

    /// Chat with report
    pub async fn chat_with_report(
        &self,
        message: &str,
        history: &[serde_json::Value],
    ) -> Result<serde_json::Value, String> {
        self.post(
            "/api/report/chat",
            serde_json::json!({
                "message": message,
                "history": history,
            }),
        )
        .await
    }

    /// Interview an agent
    pub async fn interview_agent(
        &self,
        agent_id: u64,
        message: &str,
    ) -> Result<serde_json::Value, String> {
        self.post(
            "/api/simulation/interview",
            serde_json::json!({
                "agent_id": agent_id,
                "message": message,
            }),
        )
        .await
    }

    /// List interview agents
    pub async fn list_agents(&self) -> Result<serde_json::Value, String> {
        self.get("/api/simulation/interview/agents").await
    }

    /// Get actions from simulation history
    pub async fn get_actions(
        &self,
        simulation_id: &str,
        limit: usize,
        offset: usize,
        platform: Option<&str>,
        agent_id: Option<usize>,
        round: Option<u32>,
    ) -> Result<serde_json::Value, String> {
        let mut url = format!("/api/simulation/{}/actions?limit={}&offset={}",
            simulation_id, limit, offset);
        if let Some(p) = platform {
            url.push_str(&format!("&platform={}", p));
        }
        if let Some(a) = agent_id {
            url.push_str(&format!("&agent_id={}", a));
        }
        if let Some(r) = round {
            url.push_str(&format!("&round={}", r));
        }
        self.get(&url).await
    }

    /// List projects
    pub async fn list_projects(&self) -> Result<serde_json::Value, String> {
        self.get("/api/project/list").await
    }

    /// Delete project
    pub async fn delete_project(&self, project_id: &str) -> Result<serde_json::Value, String> {
        self.post(
            "/api/project/delete",
            serde_json::json!({ "project_id": project_id }),
        )
        .await
    }

    /// Health check
    pub async fn health_check(&self) -> Result<String, String> {
        self.get_raw("/health").await
    }

    /// GET request
    async fn get(&self, path: &str) -> Result<serde_json::Value, String> {
        let url = format!("{}{}", self.base_url, path);

        let window = web_sys::window().ok_or("No window")?;
        let resp_value = JsFuture::from(window.fetch_with_str(&url))
            .await
            .map_err(|e| format!("Fetch failed: {:?}", e))?;

        let resp: web_sys::Response = resp_value.dyn_into().map_err(|_| "Invalid response")?;
        let json_val: wasm_bindgen::JsValue = JsFuture::from(resp.json().map_err(|_| "No JSON")?)
            .await
            .map_err(|e| format!("JSON parse failed: {:?}", e))?;

        let json_str = json_val.as_string().unwrap_or_default();
        serde_json::from_str(&json_str).map_err(|e| format!("Deserialize failed: {}", e))
    }

    /// GET raw text response
    async fn get_raw(&self, path: &str) -> Result<String, String> {
        let url = format!("{}{}", self.base_url, path);

        let window = web_sys::window().ok_or("No window")?;
        let resp_value = JsFuture::from(window.fetch_with_str(&url))
            .await
            .map_err(|e| format!("Fetch failed: {:?}", e))?;

        let resp: web_sys::Response = resp_value.dyn_into().map_err(|_| "Invalid response")?;
        let text_val: wasm_bindgen::JsValue = JsFuture::from(resp.text().map_err(|_| "No text")?)
            .await
            .map_err(|e| format!("Text parse failed: {:?}", e))?;

        text_val.as_string().ok_or("Invalid text".to_string())
    }

    /// POST request
    async fn post(
        &self,
        path: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let url = format!("{}{}", self.base_url, path);
        let body_str = serde_json::to_string(&body).map_err(|e| e.to_string())?;

        let mut opts = RequestInit::new();
        opts.set_method("POST");
        opts.set_body(&wasm_bindgen::JsValue::from_str(&body_str));

        let headers = web_sys::Headers::new().map_err(|_| "Headers failed")?;
        headers
            .set("Content-Type", "application/json")
            .map_err(|_| "Set header failed")?;
        opts.set_headers(&headers);

        let window = web_sys::window().ok_or("No window")?;
        let resp_value = JsFuture::from(window.fetch_with_str(&url))
            .await
            .map_err(|e| format!("Fetch failed: {:?}", e))?;

        let resp: web_sys::Response = resp_value.dyn_into().map_err(|_| "Invalid response")?;
        let json_val: wasm_bindgen::JsValue = JsFuture::from(resp.json().map_err(|_| "No JSON")?)
            .await
            .map_err(|e| format!("JSON parse failed: {:?}", e))?;

        let json_str = json_val.as_string().unwrap_or_default();
        serde_json::from_str(&json_str).map_err(|e| format!("Deserialize failed: {}", e))
    }
}