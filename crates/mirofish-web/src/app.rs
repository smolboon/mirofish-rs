//! Main Leptos application component

use leptos::prelude::*;
use leptos_router::components::{Router, Route, Routes, A};
use leptos_router::path;

use crate::components::*;

/// Main application component
#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <div class="mirofish-app">
                <nav class="app-nav">
                    <div class="nav-brand">
                        <A href="/">"MiroFish"</A>
                    </div>
                    <div class="nav-links">
                        <A href="/">"Home"</A>
                        <A href="/graph">"Graph"</A>
                        <A href="/simulation">"Simulation"</A>
                        <A href="/report">"Report"</A>
                        <A href="/interview">"Interview"</A>
                        <A href="/history">"History"</A>
                    </div>
                </nav>
                <main class="app-content">
                    <Routes fallback=|| "Page not found.".into_view()>
                        <Route path=path!("/") view=HomePage />
                        <Route path=path!("/graph") view=GraphPanel />
                        <Route path=path!("/simulation") view=SimulationPage />
                        <Route path=path!("/report") view=ReportPage />
                        <Route path=path!("/interview") view=InterviewPage />
                        <Route path=path!("/history") view=HistoryDatabase />
                    </Routes>
                </main>
            </div>
        </Router>
    }
}