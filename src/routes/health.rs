use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use crate::utils::database::DbContext;
use bson::doc;

pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({
        "status": "ok",
        "service": "Hope Backend (Rust)"
    })))
}

pub async fn readiness_check(
    State(db_context): State<Arc<DbContext>>,
) -> impl IntoResponse {
    // Ping MongoDB to check connection
    match db_context.db.run_command(doc! { "ping": 1 }, None).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "status": "ready", "db": "connected" }))),
        Err(_) => (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({ "status": "not_ready", "db": "error" }))),
    }
}
