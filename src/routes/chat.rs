use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::{IntoResponse, sse::{Event, Sse}},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::convert::Infallible;
use futures::stream::Stream;
use futures::StreamExt;
use bson::{doc, oid::ObjectId};
use chrono::{Utc, Duration};


use crate::utils::database::DbContext;
use crate::middleware::auth::AuthenticatedUser;
use crate::services::ai::AiService;
use crate::services::memory::MemoryService;
use crate::services::analysis::{AnalysisService, ToneAnalysis, ResiliencePattern};
use crate::utils::cache::CacheManager;
use crate::routes::mood::get_latest_mood;
use crate::models::resilience::ResilienceRep;

#[derive(Deserialize)]
pub struct ChatRequest {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    pub message: String,
}

#[derive(Serialize)]
pub struct ChatMetadata {
    pub technique: String,
    pub goal: String,
}

#[derive(Serialize)]
pub struct AnalysisResult {
    #[serde(rename = "emotionalState")]
    pub emotional_state: String,
    #[serde(rename = "riskLevel")]
    pub risk_level: i32,
    pub themes: Vec<String>,
    #[serde(rename = "recommendedApproach")]
    pub recommended_approach: String,
    pub tone: ToneAnalysis,
    #[serde(rename = "resiliencePatterns")]
    pub resilience_patterns: Vec<ResiliencePattern>,
    pub hypothesis: Option<String>,
    pub mantra: String,
}

#[derive(Serialize)]
pub struct ChatResponse {
    pub success: bool,
    pub response: String,
    pub suggestions: Vec<String>,
    pub analysis: AnalysisResult,
    pub metadata: ChatMetadata,
}

/// Helper to assemble the prompt using the "Turbo Lane" (RAM Cache)
async fn assemble_prompt(
    db_context: &Arc<DbContext>,
    user_id: &str,
    session_id: &str,
    message: &str
) -> (String, i32, String) {
    if let Some(session) = CacheManager::get_session(session_id) {
        if let (Some(ctx), Some(mood)) = (session.user_context, session.mood_score) {
            let tone_analysis = AnalysisService::analyze_user_tone(message);
            let tone_insight = format!("\n**User Tone Detected:** {} (Intensity: {})", 
                tone_analysis.emotion, tone_analysis.intensity);
            
            let prompt = format!(
                "User context: {}\nMood Score: {}/10\n{}\nHistory:\n{}\n\nUser: {}\nAssistant:",
                ctx, mood, tone_insight, session.history.join("\n"), message
            );
            return (prompt, mood, ctx);
        }
    }

    let history = CacheManager::get_history(session_id);
    let user_context = MemoryService::get_context(&db_context.db, user_id).await;
    let mood_score = get_latest_mood(&db_context.db, user_id).await.unwrap_or(5);

    CacheManager::cache_session_meta(session_id.to_string(), user_context.clone(), mood_score);

    let tone_analysis = AnalysisService::analyze_user_tone(message);
    let tone_insight = format!("\n**User Tone Detected:** {} (Intensity: {})", 
        tone_analysis.emotion, tone_analysis.intensity);

    let prompt = format!(
        "User context: {}\nMood Score: {}/10\n{}\nHistory:\n{}\n\nUser: {}\nAssistant:",
        user_context, mood_score, tone_insight, history.join("\n"), message
    );

    (prompt, mood_score, user_context)
}

/// ─── NEW: Background Task Orchestrator ───
/// Runs after the AI response is sent/streamed.
async fn process_background_tasks(
    db: mongodb::Database,
    ai: AiService,
    user_id: String,
    message: String,
    ai_response: String,
) {
    // 1. Detect Agreement to a Resilience Rep (Python-style proactive detection)
    if let Some((title, reason)) = ai.detect_rep_agreement(&message, &ai_response).await {
        let collection = db.collection::<ResilienceRep>("resilience_reps");
        let uid = ObjectId::parse_str(&user_id).unwrap();
        let now = bson::DateTime::from_millis(Utc::now().timestamp_millis());
        let expires = bson::DateTime::from_millis((Utc::now() + Duration::try_hours(48).unwrap()).timestamp_millis());

        let rep = ResilienceRep {
            id: None,
            userId: uid,
            title,
            reason,
            isCompleted: false,
            createdAt: now,
            expiresAt: expires,
        };

        if let Ok(_) = collection.insert_one(rep, None).await {
            tracing::info!("Auto-captured Resilience Rep for user {}", user_id);
        }
    }

    // 2. Extract Memory Facts (Zero-cost regex extraction)
    MemoryService::process_message_for_memory(&db, &user_id, &message).await;
}

pub async fn send_message(
    State(db_context): State<Arc<DbContext>>,
    user: AuthenticatedUser,
    Json(payload): Json<ChatRequest>,
) -> impl IntoResponse {
    let ai_service = &db_context.ai;
    let (prompt_base, mood_score, user_context_raw) = assemble_prompt(&db_context, &user.user_id, &payload.session_id, &payload.message).await;
    
    // 1. Psychological Dot-Connecting (The Python-style Insight Engine)
    let facts_list: Vec<String> = user_context_raw
        .lines()
        .map(|l| l.replace("- ", "").trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();

    let tone_analysis = AnalysisService::analyze_user_tone(&payload.message);
    let resilience_patterns = AnalysisService::detect_resilience_patterns(&facts_list);
    let hypothesis = AnalysisService::generate_resilience_hypothesis(&facts_list, &tone_analysis.emotion);
    let mood_label = match mood_score {
        m if m >= 8 => "happy",
        m if m >= 6 => "calm",
        m if m >= 4 => "neutral",
        m if m >= 2 => "sad",
        _ => "stressed"
    };
    let mantra = AnalysisService::generate_personalized_mantra(&facts_list, mood_label);

    // 2. Inject insights into the final prompt (Subtle/Natural Guidance)
    let insight_block = if let Some(h) = &hypothesis {
        format!(
            "\n[INTERNAL THERAPEUTIC INSIGHT (DO NOT QUOTE THIS VERBATIM): {}]\n[THEMATIC ANCHOR: {}]\n\nGuidance: Integrate this insight naturally into your voice. Do not use phrases like 'I notice that'. Just respond with this context in mind.", 
            h, mantra
        )
    } else {
        format!("\n[THEMATIC ANCHOR: {}]\nGuidance: Keep this mantra in mind but don't force it.", mantra)
    };
    
    let final_prompt = format!("{}{}\nAssistant:", prompt_base, insight_block);

    let themes = AnalysisService::extract_key_themes(&payload.message);
    let emotional_state = AnalysisService::compute_emotional_state(&payload.message, mood_score);
    
    CacheManager::update_session(payload.session_id.clone(), "User", &payload.message);

    match ai_service.generate_response(&final_prompt).await {
        Ok(ai_response) => {
            CacheManager::update_session(payload.session_id.clone(), "Hope", &ai_response);
            let suggestions = ai_service.generate_suggestions(&ai_response, &payload.message).await;

            let db = db_context.db.clone();
            let ai_clone = ai_service.clone();
            let user_id = user.user_id.clone();
            let message = payload.message.clone();
            let response_clone = ai_response.clone();
            
            tokio::spawn(async move {
                process_background_tasks(db, ai_clone, user_id, message, response_clone).await;
            });

            Json(ChatResponse {
                success: true,
                response: ai_response,
                suggestions,
                analysis: AnalysisResult {
                    emotional_state,
                    risk_level: 0,
                    themes,
                    recommended_approach: tone_analysis.recommended_mode.clone(),
                    tone: tone_analysis.clone(),
                    resilience_patterns,
                    hypothesis,
                    mantra,
                },
                metadata: ChatMetadata {
                    technique: tone_analysis.recommended_mode,
                    goal: "Provide resonance".to_string(),
                },
            })
        }
        Err(_) => {
             Json(ChatResponse {
                success: false,
                response: "Error: I'm having trouble connecting right now. Please try again.".to_string(),
                suggestions: vec![],
                 analysis: AnalysisResult {
                    emotional_state: "unknown".to_string(),
                    risk_level: 0,
                    themes: vec![],
                    recommended_approach: "supportive".to_string(),
                    tone: tone_analysis.clone(),
                    resilience_patterns: vec![],
                    hypothesis: None,
                    mantra: "I am finding my way.".to_string(),
                },
                metadata: ChatMetadata {
                    technique: "supportive".to_string(),
                    goal: "Error recovery".to_string(),
                },
            })
        }
    }
}

pub async fn chat_stream(
    State(db_context): State<Arc<DbContext>>,
    user: AuthenticatedUser,
    Json(payload): Json<ChatRequest>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let ai_service = &db_context.ai;
    let (prompt, _, _) = assemble_prompt(&db_context, &user.user_id, &payload.session_id, &payload.message).await;
    CacheManager::update_session(payload.session_id.clone(), "User", &payload.message);

    let stream = ai_service.generate_response_stream(&prompt).await;
    let db = db_context.db.clone();
    let ai_clone = ai_service.clone();
    let user_id = user.user_id.clone();
    let message = payload.message.clone();

    let event_stream = stream.map(move |result| {
        match result {
            Ok(chunk) => {
                Event::default().data(serde_json::json!({
                    "type": "chunk",
                    "content": chunk
                }).to_string())
            }
            Err(_) => Event::default().data(serde_json::json!({
                "type": "error",
                "content": "Error generating response"
            }).to_string()),
        }
    });

    // Final background tasks after stream completes
    let final_task = futures::stream::once(async move {
        tokio::spawn(async move {
            process_background_tasks(db, ai_clone, user_id, message, String::new()).await;
        });
        None::<Event>
    }).filter_map(|x| futures::future::ready(x));

    let combined_stream = event_stream.chain(final_task).map(Ok);
    Sse::new(combined_stream)
}

use crate::models::chat::ChatSession;

#[derive(Serialize)]
pub struct ChatSessionResponse {
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "sessionId")]
    pub session_id: String,
    pub title: Option<String>,
    #[serde(rename = "messageCount")]
    pub message_count: usize,
    #[serde(rename = "createdAt")]
    pub created_at: bson::DateTime,
    #[serde(rename = "updatedAt")]
    pub updated_at: bson::DateTime,
}

pub async fn list_sessions(
    State(db_context): State<Arc<DbContext>>,
    user: AuthenticatedUser,
) -> impl IntoResponse {
    let collection = db_context.db.collection::<ChatSession>("chat_sessions");
    let uid = match ObjectId::parse_str(&user.user_id) {
        Ok(id) => id,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "success": false, "message": "Invalid user ID" }))).into_response(),
    };

    let filter = doc! { "userId": uid, "isArchived": false };
    let mut cursor = match collection.find(filter, None).await {
        Ok(c) => c,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "success": false, "message": "Database error" }))).into_response(),
    };
    
    let mut sessions = Vec::new();
    while let Some(result) = cursor.next().await {
        if let Ok(session) = result {
            let id_str = session.id.map(|oid| oid.to_hex()).unwrap_or_default();
            sessions.push(ChatSessionResponse {
                id: id_str.clone(),
                session_id: id_str,
                title: session.title,
                message_count: session.messages.len(),
                created_at: session.created_at,
                updated_at: session.updated_at,
            });
        }
    }

    (StatusCode::OK, Json(serde_json::json!({ "success": true, "sessions": sessions }))).into_response()
}

pub async fn create_session(
    State(db_context): State<Arc<DbContext>>,
    user: AuthenticatedUser,
) -> impl IntoResponse {
    let collection = db_context.db.collection::<ChatSession>("chat_sessions");
    let uid = match ObjectId::parse_str(&user.user_id) {
        Ok(id) => id,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "success": false, "message": "Invalid user ID" }))).into_response(),
    };
    let now = bson::DateTime::from_millis(Utc::now().timestamp_millis());

    let new_session = ChatSession {
        id: None,
        user_id: uid,
        title: Some("New Session".to_string()),
        messages: vec![],
        created_at: now,
        updated_at: now,
        is_archived: false,
    };

    match collection.insert_one(new_session, None).await {
        Ok(result) => {
            let session_id = result.inserted_id.as_object_id().unwrap().to_hex();
            (StatusCode::CREATED, Json(serde_json::json!({ 
                "success": true, 
                "sessionId": session_id,
                "message": "Session created" 
            }))).into_response()
        },
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ 
            "success": false, 
            "message": "Failed to create session" 
        }))).into_response(),
    }
}

pub async fn delete_session(
    State(db_context): State<Arc<DbContext>>,
    user: AuthenticatedUser,
    axum::extract::Path(session_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let collection = db_context.db.collection::<ChatSession>("chat_sessions");
    let uid = match ObjectId::parse_str(&user.user_id) {
        Ok(id) => id,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "success": false, "message": "Invalid user ID" }))).into_response(),
    };
    let sid = match ObjectId::parse_str(&session_id) {
        Ok(id) => id,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "success": false, "message": "Invalid session ID" }))).into_response(),
    };

    // Soft delete
    let filter = doc! { "_id": sid, "userId": uid };
    let update = doc! { "$set": { "isArchived": true, "updatedAt": bson::DateTime::now() } };

    match collection.update_one(filter, update, None).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "success": true }))).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "success": false }))).into_response(),
    }
}
