//! Graph visualization component using Canvas

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

/// Graph visualization state
pub struct GraphViz {
    pub canvas: HtmlCanvasElement,
    pub context: CanvasRenderingContext2d,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub zoom: f64,
    pub pan_x: f64,
    pub pan_y: f64,
    pub selected_node: Option<usize>,
}

/// A node in the graph visualization
#[derive(Clone)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub entity_type: String,
    pub x: f64,
    pub y: f64,
    pub radius: f64,
    pub color: String,
}

/// An edge in the graph visualization
#[derive(Clone)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub edge_type: String,
    pub weight: f64,
}

impl GraphViz {
    pub fn new(canvas_id: &str) -> Result<Self, String> {
        let document = web_sys::window()
            .ok_or("No window")?
            .document()
            .ok_or("No document")?;

        let canvas = document
            .get_element_by_id(canvas_id)
            .ok_or(format!("Canvas element '{}' not found", canvas_id))?
            .dyn_into::<HtmlCanvasElement>()
            .map_err(|_| "Element is not a canvas")?;

        let context = canvas
            .get_context("2d")
            .map_err(|_| "Failed to get context")?
            .ok_or("No context")?
            .dyn_into::<CanvasRenderingContext2d>()
            .map_err(|_| "Not a 2d context")?;

        Ok(Self {
            canvas,
            context,
            nodes: Vec::new(),
            edges: Vec::new(),
            zoom: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
            selected_node: None,
        })
    }

    /// Set graph data
    pub fn set_graph_data(&mut self, nodes: Vec<GraphNode>, edges: Vec<GraphEdge>) {
        self.nodes = nodes;
        self.edges = edges;
        self.layout();
        self.render();
    }

    /// Simple force-directed layout
    fn layout(&mut self) {
        if self.nodes.is_empty() {
            return;
        }

        let width = self.canvas.width() as f64;
        let height = self.canvas.height() as f64;
        let center_x = width / 2.0;
        let center_y = height / 2.0;

        // Initialize positions in a circle
        let radius = (width.min(height) / 3.0) * self.zoom;
        let node_count = self.nodes.len();
        for (i, node) in self.nodes.iter_mut().enumerate() {
            let angle = (i as f64 / node_count as f64) * 2.0 * std::f64::consts::PI;
            node.x = center_x + radius * angle.cos();
            node.y = center_y + radius * angle.sin();
        }

        // Simple force simulation (10 iterations)
        for _ in 0..10 {
            // Repulsion between nodes
            for i in 0..self.nodes.len() {
                for j in (i + 1)..self.nodes.len() {
                    let dx = self.nodes[j].x - self.nodes[i].x;
                    let dy = self.nodes[j].y - self.nodes[i].y;
                    let dist = (dx * dx + dy * dy).sqrt().max(1.0);
                    let force = 100.0 / (dist * dist);
                    let fx = (dx / dist) * force;
                    let fy = (dy / dist) * force;
                    self.nodes[i].x -= fx;
                    self.nodes[i].y -= fy;
                    self.nodes[j].x += fx;
                    self.nodes[j].y += fy;
                }
            }

            // Attraction along edges
            for edge in &self.edges {
                if let (Some(source_idx), Some(target_idx)) = (
                    self.nodes.iter().position(|n| n.id == edge.source),
                    self.nodes.iter().position(|n| n.id == edge.target),
                ) {
                    let dx = self.nodes[target_idx].x - self.nodes[source_idx].x;
                    let dy = self.nodes[target_idx].y - self.nodes[source_idx].y;
                    let dist = (dx * dx + dy * dy).sqrt().max(1.0);
                    let force = (dist - 100.0) * 0.01;
                    let fx = (dx / dist) * force;
                    let fy = (dy / dist) * force;
                    self.nodes[source_idx].x += fx;
                    self.nodes[source_idx].y += fy;
                    self.nodes[target_idx].x -= fx;
                    self.nodes[target_idx].y -= fy;
                }
            }

            // Center gravity
            for node in &mut self.nodes {
                let dx = center_x - node.x;
                let dy = center_y - node.y;
                node.x += dx * 0.01;
                node.y += dy * 0.01;
            }
        }
    }

    /// Render the graph
    pub fn render(&self) {
        let width = self.canvas.width() as f64;
        let height = self.canvas.height() as f64;

        // Clear canvas
        self.context.clear_rect(0.0, 0.0, width, height);

        // Draw edges
        for edge in &self.edges {
            if let (Some(source), Some(target)) = (
                self.nodes.iter().find(|n| n.id == edge.source),
                self.nodes.iter().find(|n| n.id == edge.target),
            ) {
                self.context.begin_path();
                self.context.move_to(source.x, source.y);
                self.context.line_to(target.x, target.y);
                self.context.set_stroke_style(&JsValue::from_str("#888"));
                self.context.set_line_width(1.0);
                self.context.stroke();
            }
        }

        // Draw nodes
        for node in &self.nodes {
            self.context.begin_path();
            self.context.arc(node.x, node.y, node.radius, 0.0, 2.0 * std::f64::consts::PI);
            self.context.set_fill_style(&JsValue::from_str(&node.color));
            self.context.fill();

            // Draw label
            self.context.set_fill_style(&JsValue::from_str("#000"));
            self.context.set_font("12px sans-serif");
            self.context.fill_text(&node.label, node.x - 20.0, node.y + node.radius + 15.0);
        }
    }

    /// Zoom in
    pub fn zoom_in(&mut self) {
        self.zoom = (self.zoom * 1.2).min(5.0);
        self.layout();
        self.render();
    }

    /// Zoom out
    pub fn zoom_out(&mut self) {
        self.zoom = (self.zoom / 1.2).max(0.2);
        self.layout();
        self.render();
    }

    /// Get color for entity type
    pub fn get_type_color(entity_type: &str) -> String {
        match entity_type {
            "person" => "#4CAF50",
            "organization" => "#2196F3",
            "event" => "#FF9800",
            "concept" => "#9C27B0",
            "location" => "#F44336",
            _ => "#607D8B",
        }
        .to_string()
    }
}