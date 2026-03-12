use serde::{Deserialize, Serialize};
use bson::oid::ObjectId;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub email: String,
    pub password: String,
    #[serde(rename = "isEmailVerified")]
    pub is_email_verified: bool,
    // Add other fields from Python model as needed
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub userId: String,
    pub exp: usize,
}
