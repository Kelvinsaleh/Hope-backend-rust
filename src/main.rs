use axum::{routing::{get, post, put, delete}, Router};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod routes;
mod models;
mod services;
mod middleware;
mod config;
mod utils;
mod core;

use crate::config::Config;
use crate::utils::database::DbContext;
use crate::services::jobs::JobService;

#[tokio::main]
async fn main() {
    // 1. Initialize Logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Hope Backend (Full Rust Edition)...");

    // 2. Load Configuration
    dotenvy::dotenv().ok();
    let config = Config::from_env();
    let addr = format!("0.0.0.0:{}", config.port);

    // 3. Initialize Database
    let db_context = DbContext::init(&config.mongodb_uri).await;
    tracing::info!("Connected to MongoDB");

    // 4. Start Background Workers
    JobService::start_background_workers(db_context.clone()).await;

    // 5. Setup CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 6. Build Router
    let app = Router::new()
        .route("/health", get(routes::health::health_check))
        .route("/ready", get(routes::health::readiness_check))
        // Auth
        .route("/auth/login", post(routes::auth::login))
        // Chat
        .route("/chat", post(routes::chat::send_message))
        .route("/chat/stream", post(routes::chat::chat_stream))
        .route("/chat/sessions", get(routes::chat::list_sessions).post(routes::chat::create_session))
        .route("/chat/sessions/:id", delete(routes::chat::delete_session))
        // Journal
        .route("/journal", post(routes::journal::create_entry).get(routes::journal::list_entries))
        // Mood
        .route("/mood", post(routes::mood::track_mood).get(routes::mood::get_history))
        // Action Lab
        .route("/resilience/active", get(routes::resilience::get_active_reps))
        .route("/resilience/:id/complete", put(routes::resilience::complete_rep))
        // Grandma's Pot
        .route("/wisdom/daily", get(routes::wisdom::get_daily_wisdom))
        // Notifications
        .route("/notifications/unread-count", get(routes::notifications::get_unread_count))
        .route("/notifications", get(routes::notifications::list_notifications))
        .route("/notifications/:id/read", put(routes::notifications::mark_as_read))
        // Subscriptions
        .route("/subscriptions/status", get(routes::subscription::get_status))
        .route("/subscriptions/checkout", post(routes::subscription::create_checkout))
        // User & Settings
        .route("/user/profile", get(routes::user::get_profile).put(routes::user::update_profile))
        .route("/safety/resources", get(routes::safety::get_safety_resources))
        .route("/activity/log", post(routes::activity::log_activity))
        // Analytics & Real-time
        .route("/analytics/summary", get(routes::analytics::get_summary))
        .route("/ws", get(routes::ws::ws_handler))
        .layer(cors)
        .with_state(db_context);

    // 7. Start Server
    tracing::info!("Server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
