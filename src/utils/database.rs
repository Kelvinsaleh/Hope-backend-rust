use mongodb::{Client, Database};
use std::sync::Arc;
use crate::services::ai::AiService;

pub struct DbContext {
    pub db: Database,
    pub ai: AiService,
}

impl DbContext {
    pub async fn init(uri: &str) -> Arc<Self> {
        let client = Client::with_uri_str(uri)
            .await
            .expect("Failed to connect to MongoDB");
        
        // Use the same database name as Python
        let db = client.database("HOPE-AI");
        let ai = AiService::new();
        
        Arc::new(Self { db, ai })
    }
}
