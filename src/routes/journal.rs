use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Serialize;
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
    let uid = ObjectId::parse_str(&user.user_id).unwrap();

    let entry = JournalEntry {
        id: None,
        userId: uid,
        title: payload.title.unwrap_or_else(|| "Untitled Entry".to_string()),
        content: payload.content,
        mood: payload.mood,
        tags: payload.tags.unwrap_or_default(),
        insights: Vec::new(), // To be populated by AI service
        emotionalState: "neutral".to_string(),
        keyThemes: Vec::new(),
        concerns: Vec::new(),
        achievements: Vec::new(),
        createdAt: now,
        updatedAt: now,
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
    let uid = ObjectId::parse_str(&user.user_id).unwrap();

    let filter = doc! { "userId": uid };
    let mut cursor = collection.find(filter, None).await.unwrap();
    let mut entries = Vec::new();

    while let Some(result) = futures::StreamExt::next(&mut cursor).await {
        if let Ok(entry) = result {
            entries.push(entry);
        }
    }

    (StatusCode::OK, Json(serde_json::json!({
        "success": true,
        "entries": entries
    })))
}
