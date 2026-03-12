use axum::{
    extract::{State, Json, Path},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;
use crate::utils::database::DbContext;
use crate::middleware::auth::AuthenticatedUser;
use crate::models::notifications::Notification;
use bson::{doc, oid::ObjectId};
use futures::StreamExt;

pub async fn get_unread_count(
    State(db_context): State<Arc<DbContext>>,
    user: AuthenticatedUser,
) -> impl IntoResponse {
    let collection = db_context.db.collection::<Notification>("notifications");
    let uid = ObjectId::parse_str(&user.user_id).unwrap();

    let count = collection.count_documents(doc! { "userId": uid, "isRead": false }, None).await.unwrap();

    (StatusCode::OK, Json(serde_json::json!({ "success": true, "count": count })))
}

pub async fn list_notifications(
    State(db_context): State<Arc<DbContext>>,
    user: AuthenticatedUser,
) -> impl IntoResponse {
    let collection = db_context.db.collection::<Notification>("notifications");
    let uid = ObjectId::parse_str(&user.user_id).unwrap();

    let mut cursor = collection.find(doc! { "userId": uid }, None).await.unwrap();
    let mut list = Vec::new();

    while let Some(result) = cursor.next().await {
        if let Ok(n) = result {
            list.push(n);
        }
    }

    (StatusCode::OK, Json(serde_json::json!({ "success": true, "data": list })))
}

pub async fn mark_as_read(
    State(db_context): State<Arc<DbContext>>,
    user: AuthenticatedUser,
    Path(nid): Path<String>,
) -> impl IntoResponse {
    let collection = db_context.db.collection::<Notification>("notifications");
    let uid = ObjectId::parse_str(&user.user_id).unwrap();
    let id = ObjectId::parse_str(&nid).unwrap();

    let _ = collection.update_one(
        doc! { "_id": id, "userId": uid },
        doc! { "$set": { "isRead": true } },
        None
    ).await;

    (StatusCode::OK, Json(serde_json::json!({ "success": true })))
}
