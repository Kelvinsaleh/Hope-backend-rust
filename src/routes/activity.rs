use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;
use crate::utils::database::DbContext;
use crate::middleware::auth::AuthenticatedUser;
use crate::models::activity::UserActivity;
use bson::{oid::ObjectId, DateTime};
use chrono::Utc;

pub async fn log_activity(
    State(db_context): State<Arc<DbContext>>,
    user: AuthenticatedUser,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let collection = db_context.db.collection::<UserActivity>("user_activities");
    let uid = ObjectId::parse_str(&user.user_id).unwrap();
    let action = payload.get("action").and_then(|a| a.as_str()).unwrap_or("unknown");

    let entry = UserActivity {
        id: None,
        userId: uid,
        action: action.to_string(),
        timestamp: DateTime::from_millis(Utc::now().timestamp_millis()),
    };

    match collection.insert_one(entry, None).await {
        Ok(_) => (StatusCode::CREATED, Json(serde_json::json!({ "success": true }))),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "success": false }))),
    }
}
