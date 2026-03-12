use axum::{
    extract::{State, Query},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use crate::utils::database::DbContext;
use crate::middleware::auth::AuthenticatedUser;
use bson::{doc, oid::ObjectId};
use futures::StreamExt;

#[derive(Deserialize)]
pub struct AnalyticsQuery {
    pub period: Option<i32>,
}

pub async fn get_summary(
    State(db_context): State<Arc<DbContext>>,
    user: AuthenticatedUser,
    Query(query): Query<AnalyticsQuery>,
) -> impl IntoResponse {
    let collection = db_context.db.collection::<serde_json::Value>("journal_entries");
    let uid = ObjectId::parse_str(&user.user_id).unwrap();
    let days = query.period.unwrap_or(30);

    // 1. Calculate Average Mood
    let pipeline = vec![
        doc! { "$match": { "userId": uid } },
        doc! { "$group": { "_id": null, "avg": { "$avg": "$mood" }, "count": { "$sum": 1 } } },
    ];

    let mut cursor = collection.aggregate(pipeline, None).await.unwrap();
    let mut average = 5.0;
    let mut count = 0;

    if let Some(result) = cursor.next().await {
        if let Ok(doc) = result {
            average = doc.get_f64("avg").unwrap_or(5.0);
            count = doc.get_i32("count").unwrap_or(0);
        }
    }

    (StatusCode::OK, Json(serde_json::json!({
        "success": true,
        "data": {
            "averageMood": (average * 10.0).round() / 10.0,
            "totalEntries": count,
            "period": days,
            "message": "Insights calculated at high-speed"
        }
    })))
}
