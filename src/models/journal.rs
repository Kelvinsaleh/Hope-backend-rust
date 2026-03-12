use serde::{Deserialize, Serialize};
use bson::{oid::ObjectId, DateTime};

#[derive(Debug, Serialize, Deserialize)]
pub struct JournalEntry {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub userId: ObjectId,
    pub title: String,
    pub content: String,
    pub mood: i32,
    pub tags: Vec<String>,
    pub insights: Vec<String>,
    pub emotionalState: String,
    pub keyThemes: Vec<String>,
    pub concerns: Vec<String>,
    pub achievements: Vec<String>,
    pub createdAt: DateTime,
    pub updatedAt: DateTime,
}

#[derive(Deserialize)]
pub struct JournalCreate {
    pub title: Option<String>,
    pub content: String,
    pub mood: i32,
    pub tags: Option<Vec<String>>,
}
