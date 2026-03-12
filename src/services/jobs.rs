use std::time::Duration;
use tokio::time::sleep;
use crate::utils::database::DbContext;
use std::sync::Arc;
use crate::utils::ws_hub::WsHub;
use crate::services::ai::AiService;
use crate::services::memory::MemoryService;
use bson::doc;
use futures::StreamExt;

pub struct JobService;

impl JobService {
    pub async fn start_background_workers(db_context: Arc<DbContext>) {
        tracing::info!("Starting Rust Background Workers...");
        
        // 1. Worker for "Resonant Nudges" (AI Dreaming)
        let db_nudges = db_context.clone();
        tokio::spawn(async move {
            loop {
                // Wait for 24 hours (or whatever frequency you want)
                // For testing, let's say every 1 hour
                sleep(Duration::from_secs(3600)).await;
                Self::send_resonant_nudges(&db_nudges).await;
            }
        });

        // 2. Worker for "Cache Cleanup"
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(600)).await; // Every 10 mins
                crate::utils::cache::CacheManager::cleanup();
            }
        });
    }

    async fn send_resonant_nudges(db_context: &Arc<DbContext>) {
        tracing::info!("Running AI Dreaming (Resonant Nudges) in Rust...");
        let ai_service = AiService::new();
        let collection = db_context.db.collection::<serde_json::Value>("users");
        
        let mut cursor = collection.find(doc! { "isEmailVerified": true }, None).await.unwrap();

        while let Some(result) = cursor.next().await {
            if let Ok(user) = result {
                let uid = user.get("_id").unwrap().as_object_id().unwrap().to_hex();
                
                // Fetch user memory
                let context = MemoryService::get_context(&db_context.db, &uid).await;
                if context.is_empty() { continue; }

                // Generate Nudge
                let prompt = format!(
                    "You are Hope. Based on these user facts: [{}], write a single, soulful 10-word check-in nudge.",
                    context
                );

                if let Ok(nudge) = ai_service.generate_response(&prompt).await {
                    // PUSH via WebSocket if user is online!
                    WsHub::send_to_user(&uid, &nudge);
                    
                    // Also save to DB for offline viewing
                    let notif_coll = db_context.db.collection::<bson::Document>("notifications");
                    let _ = notif_coll.insert_one(doc! {
                        "userId": user.get("_id").unwrap(),
                        "title": "Hope check-in",
                        "message": nudge,
                        "type": "nudge",
                        "isRead": false,
                        "createdAt": bson::DateTime::now()
                    }, None).await;
                }
            }
        }
    }
}
