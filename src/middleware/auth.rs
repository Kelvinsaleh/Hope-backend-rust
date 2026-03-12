use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use crate::models::user::Claims;
use crate::utils::cache::CacheManager;

pub struct AuthenticatedUser {
    pub user_id: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {     
        // Extract Authorization header
        let auth_header = parts.headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or((StatusCode::UNAUTHORIZED, "Missing authorization header"))?;

        if !auth_header.starts_with("Bearer ") {
            return Err((StatusCode::UNAUTHORIZED, "Invalid authorization header"));
        }

        let token = &auth_header[7..];

        // 1. Try High-Speed RAM Cache (Fast Lane)
        if let Some(user_id) = CacheManager::get_user_id(token) {
            return Ok(AuthenticatedUser { user_id });
        }

        // 2. Fallback to JWT Decode (Regular Lane)
        let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");

        let mut validation = Validation::default();
        validation.leeway = 60; // 60 seconds leeway for clock skew

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_ref()),
            &validation,
        ).map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token"))?;

        let user_id = token_data.claims.userId;

        // 3. Populate RAM cache for next time
        CacheManager::cache_token(token.to_string(), user_id.clone(), 3600);

        Ok(AuthenticatedUser {
            user_id,
        })
    }
}
