use serde::{Deserialize, Serialize};
use bson::{oid::ObjectId, DateTime};

#[derive(Debug, Serialize, Deserialize)]
pub struct Subscription {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub userId: ObjectId,
    pub status: String, // "active", "expired", "pending"
    pub tier: String,   // "free", "premium"
    pub plan: String,   // "monthly", "yearly"
    pub expiresAt: DateTime,
    pub createdAt: DateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Notification {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub userId: ObjectId,
    pub title: String,
    pub message: String,
    #[serde(rename = "type")]
    pub n_type: String, // "nudge", "report", "system"
    pub isRead: bool,
    pub createdAt: DateTime,
}
