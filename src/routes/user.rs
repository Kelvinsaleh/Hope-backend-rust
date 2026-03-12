use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;
use crate::utils::database::DbContext;
use crate::middleware::auth::AuthenticatedUser;
use crate::models::user::User;
use crate::models::activity::ProfileUpdate;
use bson::{doc, oid::ObjectId, Document};

pub async fn get_profile(
    State(db_context): State<Arc<DbContext>>,
    user: AuthenticatedUser,
) -> impl IntoResponse {
    let collection = db_context.db.collection::<Document>("users");
    let uid = ObjectId::parse_str(&user.user_id).unwrap();

    let user_doc = collection.find_one(doc! { "_id": uid }, None).await.unwrap();

    match user_doc {
        Some(mut u) => {
            // Convert _id to hex string for Flutter
            if let Some(id) = u.get_object_id("_id").ok() {
                u.insert("_id", id.to_hex());
            }
            (StatusCode::OK, Json(serde_json::json!({ "success": true, "user": u })))
        },
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "success": false, "message": "User not found" }))),
    }
}

pub async fn update_profile(
    State(db_context): State<Arc<DbContext>>,
    user: AuthenticatedUser,
    Json(payload): Json<ProfileUpdate>,
) -> impl IntoResponse {
    let collection = db_context.db.collection::<serde_json::Value>("users");
    let uid = ObjectId::parse_str(&user.user_id).unwrap();

    let mut update_doc = doc! {};
    if let Some(name) = payload.name { update_doc.insert("name", name); }
    if let Some(style) = payload.preferredStyle { update_doc.insert("preferredStyle", style); }

    match collection.update_one(doc! { "_id": uid }, doc! { "$set": update_doc }, None).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "success": true }))),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "success": false }))),
    }
}
