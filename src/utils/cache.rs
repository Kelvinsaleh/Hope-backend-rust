use dashmap::DashMap;
use std::time::{Instant, Duration};
use lazy_static::lazy_static;

pub struct TokenInfo {
    pub user_id: String,
    pub expiry: Instant,
}

#[derive(Clone)]
pub struct ChatSession {
    pub history: Vec<String>,
    pub user_context: Option<String>,
    pub mood_score: Option<i32>,
    pub last_access: Instant,
}

lazy_static! {
    pub static ref TOKEN_CACHE: DashMap<String, TokenInfo> = DashMap::new();
    pub static ref SESSION_CACHE: DashMap<String, ChatSession> = DashMap::new();
}

pub struct CacheManager;

impl CacheManager {
    // ─── Token Cache ───
    pub fn cache_token(token: String, user_id: String, ttl_secs: u64) {
        TOKEN_CACHE.insert(token, TokenInfo {
            user_id,
            expiry: Instant::now() + Duration::from_secs(ttl_secs),
        });
    }

    pub fn get_user_id(token: &str) -> Option<String> {
        if let Some(info) = TOKEN_CACHE.get(token) {
            if info.expiry > Instant::now() {
                return Some(info.user_id.clone());
            }
        }
        None
    }

    // ─── Session Cache (The "Turbo Lane") ───
    
    /// Update or create session, appending history and keeping it in RAM.
    pub fn update_session(sid: String, role: &str, content: &str) {
        let mut session = SESSION_CACHE.entry(sid).or_insert(ChatSession {
            history: Vec::new(),
            user_context: None,
            mood_score: None,
            last_access: Instant::now(),
        });
        
        session.history.push(format!("{}: {}", role, content));
        if session.history.len() > 10 {
            session.history.remove(0);
        }
        session.last_access = Instant::now();
    }

    /// Store expensive metadata (mood/facts) in RAM so we don't hit DB every message.
    pub fn cache_session_meta(sid: String, context: String, mood: i32) {
        if let Some(mut session) = SESSION_CACHE.get_mut(&sid) {
            session.user_context = Some(context);
            session.mood_score = Some(mood);
            session.last_access = Instant::now();
        }
    }

    /// Retrieve the full session state for prompt assembly.
    pub fn get_session(sid: &str) -> Option<ChatSession> {
        SESSION_CACHE.get(sid).map(|s| s.clone())
    }

    pub fn get_history(sid: &str) -> Vec<String> {
        SESSION_CACHE.get(sid).map(|s| s.history.clone()).unwrap_or_default()
    }

    pub fn cleanup() {
        TOKEN_CACHE.retain(|_, v| v.expiry > Instant::now());
        SESSION_CACHE.retain(|_, v| v.last_access.elapsed() < Duration::from_secs(3600));
    }
}
