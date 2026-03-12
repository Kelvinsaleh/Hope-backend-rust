use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::utils::database::DbContext;
use crate::services::auth::AuthService;
use crate::models::user::User;
use bson::doc;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub success: bool,
    pub token: String,
    pub message: String,
}

pub async fn login(
    State(db_context): State<Arc<DbContext>>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    let collection = db_context.db.collection::<User>("users");
    
    let user = collection.find_one(doc! { "email": &payload.email }, None).await.unwrap();

    match user {
        Some(u) => {
            if AuthService::verify_password(&payload.password, &u.password) {
                let secret = std::env::var("JWT_SECRET").unwrap();
                let token = AuthService::create_token(&u.id.unwrap().to_hex(), &secret);
                
                (StatusCode::OK, Json(AuthResponse {
                    success: true,
                    token,
                    message: "Login successful".to_string(),
                }))
            } else {
                (StatusCode::UNAUTHORIZED, Json(AuthResponse {
                    success: false,
                    token: "".to_string(),
                    message: "Invalid credentials".to_string(),
                }))
            }
        }
        None => {
            (StatusCode::UNAUTHORIZED, Json(AuthResponse {
                success: false,
                token: "".to_string(),
                message: "User not found".to_string(),
            }))
        }
    }
}
