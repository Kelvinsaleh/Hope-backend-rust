use serde::{Deserialize, Serialize};
use bson::{oid::ObjectId, DateTime};

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatSession {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    #[serde(rename = "userId")]
    pub user_id: ObjectId,
    pub title: Option<String>,
    pub messages: Vec<ChatMessage>,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime,
    #[serde(rename = "isArchived", default)]
    pub is_archived: bool,
}
