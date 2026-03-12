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
            .and_then(|h| h.to_str().ok());
            
        if auth_header.is_none() {
            tracing::warn!("Auth middleware: Missing Authorization header");
            return Err((StatusCode::UNAUTHORIZED, "Missing authorization header"));
        }
        
        let auth_header = auth_header.unwrap();

        if !auth_header.starts_with("Bearer ") {
            tracing::warn!("Auth middleware: Invalid Authorization header format (must start with Bearer)");
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

        let token_data = match decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_ref()),
            &validation,
        ) {
            Ok(data) => data,
            Err(e) => {
                tracing::error!("Auth middleware: JWT decode failed for token: {}. Error: {:?}", 
                    if token.len() > 10 { format!("{}...", &token[..10]) } else { "short-token".to_string() },
                    e);
                return Err((StatusCode::UNAUTHORIZED, "Invalid token"));
            }
        };

        let user_id = token_data.claims.userId;
        tracing::debug!("Auth middleware: Successfully authenticated user {}", user_id);

        // 3. Populate RAM cache for next time
        CacheManager::cache_token(token.to_string(), user_id.clone(), 3600);

        Ok(AuthenticatedUser {
            user_id,
        })
    }
}
