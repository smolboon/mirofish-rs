//! Advanced Graph Panel component - interactive knowledge graph visualization (Leptos)

use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::CanvasRenderingContext2d;
use std::collections::HashMap;

/// Graph node representation for visualization
#[derive(Debug, Clone)]
pub struct GraphNode {
    pub id: String,
    pub name: String,
    pub entity_type: String,
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub radius: f64,
}

/// Graph edge representation for visualization
#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub label: String,
}

/// Entity type colors
const ENTITY_COLORS: &[(&str, &str)] = &[
    ("character", "#4A90D9"),
    ("event", "#E74C3C"),
    ("location", "#2ECC71"),
    ("organization", "#F39C12"),
    ("concept", "#9B59B6"),
    ("time", "#1ABC9C"),
];

/// Get color for entity type
fn get_entity_color(entity_type: &str) -> &'static str {
    for (et, color) in ENTITY_COLORS {
        if entity_type.to_lowercase().contains(et) {
            return color;
        }
    }
    "#95A5A6"
}

/// Graph panel component
#[component]
pub fn GraphPanel() -> impl IntoView {
    let canvas_ref = NodeRef::new();
    let nodes: RwSignal<Vec<GraphNode>> = RwSignal::new(vec![]);
    let edges: RwSignal<Vec<GraphEdge>> = RwSignal::new(vec![]);
    let selected_node: RwSignal<Option<usize>> = RwSignal::new(None);
    let hovered_node: RwSignal<Option<usize>> = RwSignal::new(None);
    let zoom: RwSignal<f64> = RwSignal::new(1.0);
    let pan_x: RwSignal<f64> = RwSignal::new(0.0);
    let pan_y: RwSignal<f64> = RwSignal::new(0.0);
    let node_count = move || nodes.get().len();
    let edge_count = move || edges.get().len();

    // Draw the graph
    let draw = move || {
        let canvas_el: Option<web_sys::HtmlCanvasElement> = canvas_ref.get();
        if let Some(canvas) = canvas_el {
            let ctx_result = canvas.get_context("2d");
            let ctx_opt: Option<CanvasRenderingContext2d> = match ctx_result {
                Ok(Some(js)) => js.dyn_into().ok(),
                _ => None,
            };

            if let Some(ctx) = ctx_opt {
                let width = canvas.width() as f64;
                let height = canvas.height() as f64;

                ctx.clear_rect(0.0, 0.0, width, height);
                ctx.save();
                ctx.translate(pan_x.get(), pan_y.get());
                ctx.scale(zoom.get(), zoom.get());

                let nodes_data = nodes.get();
                let edges_data = edges.get();
                let node_map: HashMap<String, usize> = nodes_data
                    .iter()
                    .enumerate()
                    .map(|(i, n)| (n.id.clone(), i))
                    .collect();

                // Draw edges
                for edge in &edges_data {
                    if let (Some(&src_idx), Some(&tgt_idx)) =
                        (node_map.get(&edge.source), node_map.get(&edge.target))
                    {
                        let src = &nodes_data[src_idx];
                        let tgt = &nodes_data[tgt_idx];
                        ctx.begin_path();
                        ctx.move_to(src.x, src.y);
                        ctx.line_to(tgt.x, tgt.y);
                        ctx.set_stroke_style(&JsValue::from_str("rgba(0, 0, 0, 0.15)"));
                        ctx.set_line_width(1.0);
                        ctx.stroke();
                    }
                }

                // Draw nodes
                let selected = selected_node.get();
                let hovered = hovered_node.get();
                for (i, node) in nodes_data.iter().enumerate() {
                    let color = get_entity_color(&node.entity_type);
                    let is_selected = selected == Some(i);
                    let is_hovered = hovered == Some(i);

                    // Shadow
                    ctx.begin_path();
                    ctx.arc(node.x + 2.0, node.y + 2.0, node.radius, 0.0, std::f64::consts::TAU)
                        .ok();
                    ctx.set_fill_style(&JsValue::from_str("rgba(0, 0, 0, 0.2)"));
                    ctx.fill();

                    // Node
                    ctx.begin_path();
                    ctx.arc(node.x, node.y, node.radius, 0.0, std::f64::consts::TAU)
                        .ok();
                    if is_selected {
                        ctx.set_fill_style(&JsValue::from_str("#FFD700"));
                    } else {
                        ctx.set_fill_style(&JsValue::from_str(color));
                    }
                    ctx.fill();

                    // Border
                    let stroke = if is_hovered {
                        "rgba(0,0,0,0.8)"
                    } else {
                        "rgba(0,0,0,0.3)"
                    };
                    ctx.set_stroke_style(&JsValue::from_str(stroke));
                    ctx.set_line_width(if is_hovered { 3.0 } else { 1.5 });
                    ctx.stroke();

                    // Label
                    ctx.set_fill_style(&JsValue::from_str("rgba(0, 0, 0, 0.8)"));
                    ctx.set_font("11px sans-serif");
                    ctx.set_text_align("center");
                    ctx.fill_text(&node.name, node.x, node.y + node.radius + 14.0)
                        .ok();
                }

                ctx.restore();
            }
        }
    };

    // Run layout iteration
    let run_layout = move || {
        let mut nodes_data = nodes.get();
        let edges_data = edges.get();
        let node_map: HashMap<String, usize> = nodes_data
            .iter()
            .enumerate()
            .map(|(i, n)| (n.id.clone(), i))
            .collect();

        let repulsion = 500.0;
        let attraction = 0.01;

        for _ in 0..10 {
            // Repulsion
            for i in 0..nodes_data.len() {
                for j in (i + 1)..nodes_data.len() {
                    let dx = nodes_data[j].x - nodes_data[i].x;
                    let dy = nodes_data[j].y - nodes_data[i].y;
                    let dist = (dx * dx + dy * dy).sqrt().max(1.0);
                    let force = repulsion / (dist * dist);
                    let fx = dx / dist * force;
                    let fy = dy / dist * force;
                    nodes_data[i].vx -= fx;
                    nodes_data[i].vy -= fy;
                    nodes_data[j].vx += fx;
                    nodes_data[j].vy += fy;
                }
            }

            // Attraction
            for edge in &edges_data {
                if let (Some(&src_idx), Some(&tgt_idx)) =
                    (node_map.get(&edge.source), node_map.get(&edge.target))
                {
                    let dx = nodes_data[tgt_idx].x - nodes_data[src_idx].x;
                    let dy = nodes_data[tgt_idx].y - nodes_data[src_idx].y;
                    let dist = (dx * dx + dy * dy).sqrt();
                    let fx = dx * dist * attraction;
                    let fy = dy * dist * attraction;
                    nodes_data[src_idx].vx += fx;
                    nodes_data[src_idx].vy += fy;
                    nodes_data[tgt_idx].vx -= fx;
                    nodes_data[tgt_idx].vy -= fy;
                }
            }

            // Apply velocities
            for node in &mut nodes_data {
                node.vx *= 0.85;
                node.vy *= 0.85;
                node.x += node.vx;
                node.y += node.vy;
            }
        }

        nodes.set(nodes_data);
        draw();
    };

    // Zoom controls
    let zoom_in = move || {
        zoom.update(|z| *z *= 1.2);
        draw();
    };
    let zoom_out = move || {
        zoom.update(|z| *z /= 1.2);
        draw();
    };
    let reset_view = move || {
        zoom.set(1.0);
        pan_x.set(0.0);
        pan_y.set(0.0);
        draw();
    };

    view! {
        <div class="graph-panel">
            <div class="graph-toolbar">
                <button class="btn btn-icon" on:click=move |_| zoom_in() title="Zoom In">"+" </button>
                <button class="btn btn-icon" on:click=move |_| zoom_out() title="Zoom Out">"-" </button>
                <button class="btn btn-secondary" on:click=move |_| reset_view()>"Reset View"</button>
                <button class="btn btn-outline" on:click=move |_| run_layout()>"Run Layout"</button>
                <span class="graph-info">
                    {move || format!("{} nodes, {} edges", node_count(), edge_count())}
                </span>
            </div>
            <canvas
                node_ref=canvas_ref
                width=800
                height=600
                class="graph-canvas"
            />
        </div>
    }
}