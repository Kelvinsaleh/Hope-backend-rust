use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;
use crate::utils::database::DbContext;
use crate::middleware::auth::AuthenticatedUser;
use crate::models::journal::{JournalEntry, JournalCreate};
use bson::{doc, oid::ObjectId, DateTime};
use chrono::Utc;

pub async fn create_entry(
    State(db_context): State<Arc<DbContext>>,
    user: AuthenticatedUser,
    Json(payload): Json<JournalCreate>,
) -> impl IntoResponse {
    let collection = db_context.db.collection::<JournalEntry>("journal_entries");
    
    let now = DateTime::from_millis(Utc::now().timestamp_millis());
    let uid = match ObjectId::parse_str(&user.user_id) {
        Ok(id) => id,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "success": false, "message": "Invalid user ID" }))).into_response(),
    };

    let entry = JournalEntry {
        id: None,
        user_id: uid,
        title: payload.title.unwrap_or_else(|| "Untitled Entry".to_string()),
        content: payload.content,
        mood: payload.mood,
        tags: payload.tags.unwrap_or_default(),
        insights: Vec::new(),
        emotional_state: "neutral".to_string(),
        key_themes: Vec::new(),
        concerns: Vec::new(),
        achievements: Vec::new(),
        created_at: now,
        updated_at: now,
    };

    match collection.insert_one(entry, None).await {
        Ok(result) => {
            (StatusCode::CREATED, Json(serde_json::json!({
                "success": true,
                "id": result.inserted_id.as_object_id().unwrap().to_hex(),
                "message": "Journal entry created"
            })))
        }
        Err(e) => {
            tracing::error!("Failed to create journal entry: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "success": false,
                "message": format!("Database Error: {}", e)
            })))
        }
    }
}

pub async fn list_entries(
    State(db_context): State<Arc<DbContext>>,
    user: AuthenticatedUser,
) -> impl IntoResponse {
    let collection = db_context.db.collection::<JournalEntry>("journal_entries");
    let uid = match ObjectId::parse_str(&user.user_id) {
        Ok(id) => id,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "success": false, "message": "Invalid user ID" }))).into_response(),
    };

    let filter = doc! { "userId": uid };
    match collection.find(filter, None).await {
        Ok(mut cursor) => {
            let mut entries = Vec::new();
            while let Some(result) = futures::StreamExt::next(&mut cursor).await {
                if let Ok(entry) = result {
                    entries.push(entry);
                }
            }
            (StatusCode::OK, Json(serde_json::json!({
                "success": true,
                "entries": entries
            }))).into_response()
        },
        Err(e) => {
            tracing::error!("Failed to list journal entries: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "success": false,
                "message": "Database error"
            }))).into_response()
        }
    }
}
