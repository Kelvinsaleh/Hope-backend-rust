use serde::{Deserialize, Serialize};
use bson::{oid::ObjectId, DateTime};

#[derive(Debug, Serialize, Deserialize)]
pub struct JournalEntry {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    #[serde(rename = "userId")]
    pub user_id: ObjectId,
    pub title: String,
    pub content: String,
    pub mood: i32,
    pub tags: Vec<String>,
    pub insights: Vec<String>,
    #[serde(rename = "emotionalState")]
    pub emotional_state: String,
    #[serde(rename = "keyThemes")]
    pub key_themes: Vec<String>,
    pub concerns: Vec<String>,
    pub achievements: Vec<String>,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime,
}

#[derive(Deserialize)]
pub struct JournalCreate {
    pub title: Option<String>,
    pub content: String,
    pub mood: i32,
    pub tags: Option<Vec<String>>,
}
