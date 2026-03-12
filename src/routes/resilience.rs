use axum::{
    extract::{State, Json, Path},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;
use crate::utils::database::DbContext;
use crate::middleware::auth::AuthenticatedUser;
use crate::models::resilience::ResilienceRep;
use bson::{doc, oid::ObjectId, DateTime};
use chrono::{Utc, Duration};
use futures::StreamExt;

pub async fn get_active_reps(
    State(db_context): State<Arc<DbContext>>,
    user: AuthenticatedUser,
) -> impl IntoResponse {
    let collection = db_context.db.collection::<ResilienceRep>("resilience_reps");
    let uid = ObjectId::parse_str(&user.user_id).unwrap();
    
    let _now = DateTime::from_millis(Utc::now().timestamp_millis());
    let expiry_limit = DateTime::from_millis((Utc::now() - Duration::try_hours(48).unwrap()).timestamp_millis());

    let filter = doc! {
        "userId": uid,
        "isCompleted": false,
        "createdAt": { "$gte": expiry_limit }
    };

    let mut cursor = collection.find(filter, None).await.unwrap();
    let mut reps = Vec::new();

    while let Some(result) = cursor.next().await {
        if let Ok(rep) = result {
            reps.push(rep);
            if reps.len() >= 3 { break; }
        }
    }

    (StatusCode::OK, Json(serde_json::json!({ "success": true, "reps": reps })))
}

pub async fn complete_rep(
    State(db_context): State<Arc<DbContext>>,
    user: AuthenticatedUser,
    Path(rep_id): Path<String>,
) -> impl IntoResponse {
    let collection = db_context.db.collection::<ResilienceRep>("resilience_reps");
    let uid = ObjectId::parse_str(&user.user_id).unwrap();
    let rid = ObjectId::parse_str(&rep_id).unwrap();

    let filter = doc! { "_id": rid, "userId": uid };
    let update = doc! { "$set": { "isCompleted": true } };

    match collection.update_one(filter, update, None).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "success": true }))),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "success": false }))),
    }
}
