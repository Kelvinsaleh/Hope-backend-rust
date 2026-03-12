use serde::{Deserialize, Serialize};
use bson::{oid::ObjectId, DateTime};

#[derive(Debug, Serialize, Deserialize)]
pub struct ResilienceRep {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub userId: ObjectId,
    pub title: String,
    pub reason: String,
    pub isCompleted: bool,
    pub createdAt: DateTime,
    pub expiresAt: DateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DailyWisdom {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub userId: ObjectId,
    pub date: String,
    pub title: String,
    pub story: String,
    pub joke: String,
    pub createdAt: DateTime,
}
