use jsonwebtoken::{encode, Header, EncodingKey};
use bcrypt::{verify, hash, DEFAULT_COST};
use crate::models::user::Claims;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct AuthService;

impl AuthService {
    pub fn create_token(user_id: &str, secret: &str) -> String {
        let expiration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs() + 8760 * 3600; // 1 year expiry

        let claims = Claims {
            userId: user_id.to_owned(),
            exp: expiration as usize,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_ref()),
        ).expect("Failed to create token")
    }

    pub fn hash_password(password: &str) -> String {
        hash(password, DEFAULT_COST).expect("Failed to hash password")
    }

    pub fn verify_password(password: &str, hashed: &str) -> bool {
        verify(password, hashed).expect("Failed to verify password")
    }
}
