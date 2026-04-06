//! Report Agent - orchestrates report generation with ReACT pattern

use std::sync::Arc;
use tracing::{info, debug};

use mirofish_core::{Report, ReportStatus, ReportSection};
use mirofish_graph::client::ZepClient;
use mirofish_llm::LLMClient;
use mirofish_task::TaskManager;

use crate::planner::{generate_report_outline, generate_section_with_research};
use crate::react::react_loop;
use crate::tools::insight_forge;

/// Report generation agent
pub struct ReportAgent {
    llm: LLMClient,
    zep: ZepClient,
    task_manager: Arc<TaskManager>,
}

impl ReportAgent {
    pub fn new(llm: LLMClient, zep: ZepClient, task_manager: Arc<TaskManager>) -> Self {
        Self { llm, zep, task_manager }
    }

    /// Generate a complete report
    pub async fn generate_report(
        &self,
        simulation_id: &str,
        graph_id: &str,
        simulation_requirement: &str,
        task_id: &str,
    ) -> Result<Report, String> {
        info!("Starting report generation for simulation: {}", simulation_id);

        let mut report = Report::new(simulation_id, graph_id, simulation_requirement);
        report.status = ReportStatus::Planning;

        // Step 1: Get graph data for context
        self.task_manager.update_task(
            task_id,
            None,
            Some(5),
            Some("Fetching graph data..."),
            None,
        );

        let graph_data = self.zep.get_graph_data(graph_id).await
            .map_err(|e| format!("Failed to get graph data: {}", e))?;

        // Step 2: Generate outline
        self.task_manager.update_task(
            task_id,
            None,
            Some(15),
            Some("Planning report outline..."),
            None,
        );

        report.status = ReportStatus::Planning;

        // Get sample facts
        let facts = insight_forge(&self.zep, graph_id, simulation_requirement, 10).await
            .map(|r| r.facts)
            .unwrap_or_default();

        let facts_json = serde_json::to_string(&facts).unwrap_or_default();

        let outline = generate_report_outline(
            &self.llm,
            simulation_requirement,
            graph_data.node_count,
            graph_data.edge_count,
            &[],
            graph_data.node_count,
            &facts_json,
        ).await?;

        report.outline = Some(outline.clone());

        // Step 3: Generate each section
        report.status = ReportStatus::Generating;
        let total_sections = outline.sections.len();

        for (i, section_outline) in outline.sections.iter().enumerate() {
            let progress = 15 + ((i + 1) as f64 / total_sections as f64 * 80.0) as u8;
            self.task_manager.update_task(
                task_id,
                None,
                Some(progress),
                Some(&format!("Generating section: {}", section_outline.title)),
                Some(serde_json::json!({
                    "current_section": section_outline.title,
                    "section_index": i + 1,
                    "total_sections": total_sections,
                })),
            );

            debug!("Generating section {}/{}: {}", i + 1, total_sections, section_outline.title);

            // Use ReACT loop for research
            let research_result = react_loop(
                &self.llm,
                &self.zep,
                graph_id,
                &format!(
                    "You are researching for a report section. Use tools to gather evidence about: {}",
                    section_outline.title
                ),
                &section_outline.description,
                simulation_requirement,
            ).await.unwrap_or_default();

            // Generate section content
            let content = generate_section_with_research(
                &self.llm,
                &section_outline.title,
                &outline.title,
                &outline.summary,
                simulation_requirement,
                &research_result,
            ).await?;

            report.sections.push(ReportSection {
                index: i,
                title: section_outline.title.clone(),
                description: section_outline.description.clone(),
                content,
                tool_calls_count: 0,
            });
        }

        // Step 4: Compile markdown
        report.markdown_content = self.compile_markdown(&report);
        report.status = ReportStatus::Completed;
        report.progress_percent = 100;

        info!("Report generated: {} sections", report.sections.len());

        // Complete the task
        self.task_manager.complete_task(
            task_id,
            serde_json::json!({
                "report_id": report.report_id,
                "sections": report.sections.len(),
            }),
        );

        Ok(report)
    }

    /// Compile report sections into markdown
    fn compile_markdown(&self, report: &Report) -> String {
        let mut md = String::new();

        if let Some(outline) = &report.outline {
            md.push_str(&format!("# {}\n\n", outline.title));
            md.push_str(&format!("{}\n\n", outline.summary));
        }

        for section in &report.sections {
            md.push_str(&format!("## {}\n\n", section.title));
            if !section.description.is_empty() {
                md.push_str(&format!("{}\n\n", section.description));
            }
            md.push_str(&format!("{}\n\n", section.content));
        }

        md
    }
}