use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::utils::database::DbContext;
use crate::services::auth::AuthService;
use bson::{doc, Document};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub success: bool,
    pub token: String,
    pub user: Option<serde_json::Value>,
    pub message: String,
}

pub async fn login(
    State(db_context): State<Arc<DbContext>>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    let collection = db_context.db.collection::<Document>("users");
    
    let user = collection.find_one(doc! { "email": &payload.email }, None).await.unwrap();

    match user {
        Some(mut u) => {
            if AuthService::verify_password(&payload.password, u.get_str("password").unwrap()) {
                let secret = std::env::var("JWT_SECRET").unwrap();
                let uid_obj = u.get_object_id("_id").unwrap();
                let token = AuthService::create_token(&uid_obj.to_hex(), &secret);
                
                // Convert _id to hex for Flutter
                u.insert("_id", uid_obj.to_hex());
                // Remove password from response
                u.remove("password");

                (StatusCode::OK, Json(AuthResponse {
                    success: true,
                    token,
                    user: Some(serde_json::to_value(u).unwrap()),
                    message: "Login successful".to_string(),
                }))
            } else {
                (StatusCode::UNAUTHORIZED, Json(AuthResponse {
                    success: false,
                    token: "".to_string(),
                    user: None,
                    message: "Invalid credentials".to_string(),
                }))
            }
        }
        None => {
            (StatusCode::UNAUTHORIZED, Json(AuthResponse {
                success: false,
                token: "".to_string(),
                user: None,
                message: "User not found".to_string(),
            }))
        }
    }
}
