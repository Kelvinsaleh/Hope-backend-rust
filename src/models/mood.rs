use serde::{Deserialize, Serialize};
use bson::{oid::ObjectId, DateTime};

#[derive(Debug, Serialize, Deserialize)]
pub struct MoodEntry {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub userId: ObjectId,
    pub score: i32,
    pub note: Option<String>,
    pub timestamp: DateTime,
    pub source: String, // "manual" or "journal"
}

#[derive(Deserialize)]
pub struct MoodCreate {
    pub score: i32,
    pub note: Option<String>,
}
