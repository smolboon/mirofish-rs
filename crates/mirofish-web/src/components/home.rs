//! Home page component (Leptos)

use leptos::prelude::*;
use leptos_router::components::A;

/// Home page component
#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <div class="home-page">
            <div class="home-hero">
                <h1>"MiroFish"</h1>
                <p class="subtitle">"Multi-Agent Social Simulation Platform"</p>
                <p class="description">
                    "A Simple and Universal Swarm Intelligence Engine, Predicting Anything"
                </p>
            </div>

            <div class="features">
                <div class="feature-card">
                    <div class="feature-icon">"📊"</div>
                    <h3>"Knowledge Graph"</h3>
                    <p>"Build knowledge graphs from documents using LLM-powered ontology generation"</p>
                </div>
                <div class="feature-card">
                    <div class="feature-icon">"🤖"</div>
                    <h3>"Agent Simulation"</h3>
                    <p>"Simulate social media behavior with AI agents on Twitter and Reddit"</p>
                </div>
                <div class="feature-card">
                    <div class="feature-icon">"📝"</div>
                    <h3>"Report Generation"</h3>
                    <p>"Generate comprehensive reports with ReACT-enhanced analysis"</p>
                </div>
                <div class="feature-card">
                    <div class="feature-icon">"💬"</div>
                    <h3>"Agent Interview"</h3>
                    <p>"Interview simulated agents about their experiences and decisions"</p>
                </div>
            </div>

            <div class="actions">
                <A href="/graph">
                    <span class="btn btn-primary">"Create Project"</span>
                </A>
            </div>
        </div>
    }
}
