use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;
use crate::utils::database::DbContext;
use crate::middleware::auth::AuthenticatedUser;
use crate::models::mood::{MoodEntry, MoodCreate};
use bson::{doc, oid::ObjectId, DateTime};
use chrono::Utc;
use mongodb::options::FindOneOptions;

pub async fn get_latest_mood(db: &mongodb::Database, user_id: &str) -> Option<i32> {
    let collection = db.collection::<MoodEntry>("moods");
    let uid = match ObjectId::parse_str(user_id) {
        Ok(id) => id,
        Err(_) => return None,
    };

    let options = FindOneOptions::builder()
        .sort(doc! { "timestamp": -1 })
        .build();

    match collection.find_one(doc! { "userId": uid }, options).await {
        Ok(Some(entry)) => Some(entry.score),
        _ => None,
    }
}

pub async fn track_mood(
    State(db_context): State<Arc<DbContext>>,
    user: AuthenticatedUser,
    Json(payload): Json<MoodCreate>,
) -> impl IntoResponse {
    let collection = db_context.db.collection::<MoodEntry>("moods");
    let now = DateTime::from_millis(Utc::now().timestamp_millis());
    let uid = ObjectId::parse_str(&user.user_id).unwrap();

    let entry = MoodEntry {
        id: None,
        userId: uid,
        score: payload.score,
        note: payload.note,
        timestamp: now,
        source: "manual".to_string(),
    };

    match collection.insert_one(entry, None).await {
        Ok(_) => (StatusCode::CREATED, Json(serde_json::json!({ "success": true }))),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "success": false }))),
    }
}

pub async fn get_history(
    State(db_context): State<Arc<DbContext>>,
    user: AuthenticatedUser,
) -> impl IntoResponse {
    let collection = db_context.db.collection::<MoodEntry>("moods");
    let uid = ObjectId::parse_str(&user.user_id).unwrap();

    let mut cursor = collection.find(doc! { "userId": uid }, None).await.unwrap();
    let mut history = Vec::new();

    while let Some(result) = futures::StreamExt::next(&mut cursor).await {
        if let Ok(entry) = result {
            history.push(entry);
        }
    }

    (StatusCode::OK, Json(serde_json::json!({ "success": true, "history": history })))
}
