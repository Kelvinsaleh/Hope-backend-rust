use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;
use crate::utils::database::DbContext;
use crate::middleware::auth::AuthenticatedUser;
use crate::models::resilience::DailyWisdom;
use crate::services::ai::AiService;
use crate::services::memory::MemoryService;
use bson::{doc, oid::ObjectId, DateTime};
use chrono::{Utc, Duration};

pub async fn get_daily_wisdom(
    State(db_context): State<Arc<DbContext>>,
    user: AuthenticatedUser,
) -> impl IntoResponse {
    let collection = db_context.db.collection::<DailyWisdom>("daily_wisdom");
    let uid = ObjectId::parse_str(&user.user_id).unwrap();
    
    // 1. Check for 'simmering' wisdom (last 3 days)
    let three_days_ago = DateTime::from_millis((Utc::now() - Duration::days(3)).timestamp_millis());
    let filter = doc! { "userId": uid, "createdAt": { "$gte": three_days_ago } };
    
    if let Ok(Some(existing)) = collection.find_one(filter, None).await {
        return (StatusCode::OK, Json(serde_json::json!({ "success": true, "data": existing })));
    }

    // 2. Cook new wisdom
    let ai_service = AiService::new();
    let facts = MemoryService::get_context(&db_context.db, &user.user_id).await;
    
    let prompt = format!(
        "You are a wise Grandmother. Using these user facts: [{}], cook up a 3-sentence parable and a corny joke. Return ONLY JSON: {{\"story\": \"...\", \"joke\": \"...\", \"title\": \"Today's Serving\"}}",
        facts
    );

    match ai_service.generate_response(&prompt).await {
        Ok(raw_json) => {
            let data: serde_json::Value = serde_json::from_str(&raw_json).unwrap_or_else(|_| {
                serde_json::json!({ "story": "Take it slow, dear.", "joke": "Why did the cow cross the road? To get to the udder side!", "title": "Grandma's Snack" })
            });

            let new_wisdom = DailyWisdom {
                id: None,
                userId: uid,
                date: Utc::now().format("%Y-%m-%d").to_string(),
                title: data["title"].as_str().unwrap_or("Grandma's Pot").to_string(),
                story: data["story"].as_str().unwrap_or("").to_string(),
                joke: data["joke"].as_str().unwrap_or("").to_string(),
                createdAt: DateTime::from_millis(Utc::now().timestamp_millis()),
            };

            collection.insert_one(&new_wisdom, None).await.unwrap();
            (StatusCode::OK, Json(serde_json::json!({ "success": true, "data": new_wisdom })))
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "success": false })))
    }
}
