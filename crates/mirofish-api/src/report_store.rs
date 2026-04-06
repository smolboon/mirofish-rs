//! Report storage and management

use std::collections::HashMap;
use std::sync::RwLock;
use chrono::{DateTime, Utc};

/// A stored report
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StoredReport {
    pub report_id: String,
    pub simulation_id: String,
    pub status: String,
    pub sections: Vec<ReportSection>,
    pub markdown_content: String,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

/// A report section
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReportSection {
    pub index: usize,
    pub title: String,
    pub content: String,
}

impl StoredReport {
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "report_id": self.report_id,
            "simulation_id": self.simulation_id,
            "status": self.status,
            "sections": self.sections.iter().map(|s| serde_json::json!({
                "index": s.index,
                "title": s.title,
                "content": s.content,
            })).collect::<Vec<_>>(),
            "markdown_content": self.markdown_content,
            "created_at": self.created_at.to_rfc3339(),
            "completed_at": self.completed_at.map(|d| d.to_rfc3339()),
            "error": self.error,
        })
    }
}

/// In-memory report store
pub struct ReportStore {
    reports: RwLock<HashMap<String, StoredReport>>,
    simulation_to_report: RwLock<HashMap<String, String>>, // simulation_id -> report_id
}

impl ReportStore {
    pub fn new() -> Self {
        Self {
            reports: RwLock::new(HashMap::new()),
            simulation_to_report: RwLock::new(HashMap::new()),
        }
    }

    /// Save a report
    pub fn save_report(&self, report: StoredReport) {
        let sim_id = report.simulation_id.clone();
        let report_id = report.report_id.clone();
        let mut reports = self.reports.write().unwrap();
        reports.insert(report_id.clone(), report);
        drop(reports);
        
        let mut sim_map = self.simulation_to_report.write().unwrap();
        sim_map.insert(sim_id, report_id);
    }

    /// Get a report by ID
    pub fn get_report(&self, report_id: &str) -> Option<serde_json::Value> {
        let reports = self.reports.read().unwrap();
        reports.get(report_id).map(|r| r.to_json())
    }

    /// Get a report by simulation ID
    pub fn get_report_by_simulation(&self, simulation_id: &str) -> Option<serde_json::Value> {
        let sim_map = self.simulation_to_report.read().unwrap();
        let report_id = sim_map.get(simulation_id)?;
        let reports = self.reports.read().unwrap();
        reports.get(report_id).map(|r| r.to_json())
    }

    /// List reports
    pub fn list_reports(&self, limit: usize) -> Vec<serde_json::Value> {
        let reports = self.reports.read().unwrap();
        let mut report_list: Vec<_> = reports.values().map(|r| r.to_json()).collect();
        report_list.sort_by(|a, b| {
            let a_time = a.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
            let b_time = b.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
            b_time.cmp(a_time) // newest first
        });
        report_list.into_iter().take(limit).collect()
    }

    /// Delete a report
    pub fn delete_report(&self, report_id: &str) -> bool {
        let mut reports = self.reports.write().unwrap();
        if let Some(report) = reports.remove(report_id) {
            let mut sim_map = self.simulation_to_report.write().unwrap();
            sim_map.remove(&report.simulation_id);
            true
        } else {
            false
        }
    }

    /// Get report sections
    pub fn get_sections(&self, report_id: &str) -> Vec<serde_json::Value> {
        let reports = self.reports.read().unwrap();
        reports.get(report_id).map(|r| {
            r.sections.iter().map(|s| serde_json::json!({
                "filename": format!("section_{:02}.md", s.index),
                "section_index": s.index,
                "content": s.content,
            })).collect()
        }).unwrap_or_default()
    }

    /// Get a single section
    pub fn get_section(&self, report_id: &str, section_index: usize) -> Option<String> {
        let reports = self.reports.read().unwrap();
        reports.get(report_id).and_then(|r| {
            r.sections.iter()
                .find(|s| s.index == section_index)
                .map(|s| s.content.clone())
        })
    }

    /// Check if report is complete
    pub fn is_report_complete(&self, report_id: &str) -> bool {
        let reports = self.reports.read().unwrap();
        reports.get(report_id).map(|r| r.status == "completed").unwrap_or(false)
    }
}