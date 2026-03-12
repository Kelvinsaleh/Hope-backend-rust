use axum::{
    routing::{get, post, put, delete}, 
    Router,
    http::{Method, header},
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
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

    // 5. Setup Robust CORS based on Render Env Vars
    let mut cors = CorsLayer::new()
        .allow_methods([
            Method::GET, 
            Method::POST, 
            Method::PUT, 
            Method::DELETE, 
            Method::OPTIONS, 
            Method::PATCH
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
            header::ORIGIN,
        ]);

    // Handle Origins from Config
    if config.allow_localhost {
        cors = cors.allow_origin([
            "http://localhost:3000".parse().unwrap(),
            "http://127.0.0.1:3000".parse().unwrap(),
            "http://localhost:8080".parse().unwrap(),
            "http://localhost:5000".parse().unwrap(),
            "http://localhost:53503".parse().unwrap(), // common flutter web port
            config.cors_origin.parse().unwrap(),
        ]);
    } else {
        cors = cors.allow_origin(config.cors_origin.parse().unwrap());
    }

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
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(db_context);

    // 7. Start Server
    tracing::info!("Server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
