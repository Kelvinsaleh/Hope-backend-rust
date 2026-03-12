use serde::{Deserialize, Serialize};
use bson::{oid::ObjectId, DateTime};

#[derive(Debug, Serialize, Deserialize)]
pub struct Meditation {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub title: String,
    pub description: String,
    pub category: String, // "Sleep", "Focus", "Anxiety"
    pub audioUrl: String,
    pub durationMinutes: i32,
    pub createdAt: DateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MeditationLog {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub userId: ObjectId,
    pub meditationId: ObjectId,
    pub completed: bool,
    pub timestamp: DateTime,
}
