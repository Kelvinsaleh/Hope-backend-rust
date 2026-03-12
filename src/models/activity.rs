use serde::{Deserialize, Serialize};
use bson::{oid::ObjectId, DateTime};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserActivity {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub userId: ObjectId,
    pub action: String, // "chat", "journal", "meditation"
    pub timestamp: DateTime,
}

#[derive(Deserialize)]
pub struct ProfileUpdate {
    pub name: Option<String>,
    pub preferredStyle: Option<String>,
}
