use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;
use crate::utils::database::DbContext;
use crate::middleware::auth::AuthenticatedUser;
use crate::models::notifications::Subscription;
use bson::{doc, oid::ObjectId};

pub async fn get_status(
    State(db_context): State<Arc<DbContext>>,
    user: AuthenticatedUser,
) -> impl IntoResponse {
    let collection = db_context.db.collection::<Subscription>("subscriptions");
    let uid = ObjectId::parse_str(&user.user_id).unwrap();

    let subscription = collection.find_one(doc! { "userId": uid, "status": "active" }, None).await.unwrap();

    match subscription {
        Some(sub) => {
            (StatusCode::OK, Json(serde_json::json!({
                "success": true,
                "isActive": true,
                "tier": sub.tier,
                "plan": sub.plan,
                "expiresAt": sub.expiresAt
            })))
        }
        None => {
            (StatusCode::OK, Json(serde_json::json!({
                "success": true,
                "isActive": false,
                "tier": "free",
                "message": "No active subscription found"
            })))
        }
    }
}

pub async fn create_checkout(
    State(_db_context): State<Arc<DbContext>>,
    _user: AuthenticatedUser,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    // This will link to Paystack logic
    // For now, returning a mock checkout link
    let plan = payload.get("plan").and_then(|p| p.as_str()).unwrap_or("monthly");
    
    (StatusCode::OK, Json(serde_json::json!({
        "success": true,
        "checkoutUrl": "https://checkout.paystack.com/mock-link",
        "message": format!("Checkout created for {} plan", plan)
    })))
}
