use std::env;

pub struct Config {
    pub mongodb_uri: String,
    pub jwt_secret: String,
    pub gemini_api_key: String,
    pub port: u16,
    pub cors_origin: String,
    pub allow_localhost: bool,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            mongodb_uri: env::var("MONGODB_URI").expect("MONGODB_URI must be set"),
            jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            gemini_api_key: env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set"),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .expect("PORT must be a number"),
            cors_origin: env::var("CORS_ORIGIN").unwrap_or_else(|_| "*".to_string()),
            allow_localhost: env::var("ALLOW_LOCALHOST_ORIGINS")
                .map(|v| v == "true")
                .unwrap_or(false),
        }
    }
}
