use reqwest::Client;
use serde_json::json;
use crate::config::Config;
use futures::Stream;
use futures::StreamExt;
use std::pin::Pin;
use std::sync::Arc;
use dashmap::DashMap;
use chrono::{DateTime, Utc, Duration};
use crate::core::persona::get_hope_cache_content;

#[derive(Clone)]
struct CacheEntry {
    name: String,
    expires_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct AiService {
    client: Client,
    api_key: String,
    cache: Arc<DashMap<String, CacheEntry>>,
}

impl AiService {
    pub fn new() -> Self {
        let config = Config::from_env();
        Self {
            client: Client::new(),
            api_key: config.gemini_api_key,
            cache: Arc::new(DashMap::new()),
        }
    }

    async fn get_or_create_cache(&self) -> Option<String> {
        // 1. Check existing cache
        if let Some(entry) = self.cache.get("hope_core") {
            if entry.expires_at > Utc::now() + Duration::minutes(5) {
                return Some(entry.name.clone());
            }
        }

        // 2. Create new cache
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/cachedContents?key={}",
            self.api_key
        );

        let system_instruction = get_hope_cache_content();
        
        let body = json!({
            "model": "models/gemini-2.5-flash",
            "systemInstruction": {
                "parts": [{ "text": system_instruction }]
            },
            "ttl": "3600s"
        });

        match self.client.post(url).json(&body).send().await {
            Ok(resp) => {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    if let Some(name) = json["name"].as_str() {
                        let entry = CacheEntry {
                            name: name.to_string(),
                            expires_at: Utc::now() + Duration::hours(1),
                        };
                        self.cache.insert("hope_core".to_string(), entry.clone());
                        return Some(entry.name);
                    }
                }
            }
            Err(e) => {
                tracing::error!("Failed to create Gemini cache: {}", e);
            }
        }
        None
    }

    pub async fn generate_response(&self, prompt: &str) -> Result<String, reqwest::Error> {
        let cache_name = self.get_or_create_cache().await;
        
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
            self.api_key
        );

        let mut body = json!({
            "contents": [{
                "parts": [{
                    "text": prompt
                }]
            }]
        });

        if let Some(name) = cache_name {
            body["cachedContent"] = json!(name);
        }

        let response = self.client.post(url)
            .json(&body)
            .send()
            .await?;

        let json: serde_json::Value = response.json().await?;
        
        let text = json["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .unwrap_or("Error: No response generated")
            .to_string();

        Ok(text)
    }

    pub async fn generate_suggestions(&self, ai_message: &str, user_history: &str) -> Vec<String> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
            self.api_key
        );

        let prompt = format!(
            "Based on the following conversation history and the latest AI message, generate exactly 3 short, natural-sounding suggested replies the user might say next.\n\nHistory: {}\nAI: {}\n\nReturn the suggestions as a JSON array of 3 strings.",
            user_history, ai_message
        );

        let body = json!({
            "contents": [{
                "parts": [{
                    "text": prompt
                }]
            }],
            "generationConfig": {
                "response_mime_type": "application/json"
            }
        });

        let fallbacks = vec![
            "Tell me more".to_string(),
            "I see".to_string(),
            "That's interesting".to_string(),
        ];

        let response = match self.client.post(url).json(&body).send().await {
            Ok(resp) => resp,
            Err(_) => return fallbacks,
        };

        let json_value: serde_json::Value = match response.json().await {
            Ok(j) => j,
            Err(_) => return fallbacks,
        };

        let text = json_value["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .unwrap_or("");

        if text.is_empty() {
            return fallbacks;
        }

        match serde_json::from_str::<Vec<String>>(text) {
            Ok(suggestions) => {
                if suggestions.len() >= 3 {
                    suggestions.into_iter().take(3).collect()
                } else {
                    fallbacks
                }
            }
            Err(_) => fallbacks,
        }
    }

    pub async fn generate_response_stream(&self, prompt: &str) -> Pin<Box<dyn Stream<Item = Result<String, reqwest::Error>> + Send>> {
        let cache_name = self.get_or_create_cache().await;
        
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:streamGenerateContent?alt=sse&key={}",
            self.api_key
        );

        let mut body = json!({
            "contents": [{
                "parts": [{
                    "text": prompt
                }]
            }]
        });

        if let Some(name) = cache_name {
            body["cachedContent"] = json!(name);
        }

        let response_result = self.client.post(url)
            .json(&body)
            .send()
            .await;

        match response_result {
            Ok(response) => {
                let stream = response.bytes_stream().map(|item| {
                    item.map(|bytes| {
                        let text = String::from_utf8_lossy(&bytes).to_string();
                        let mut combined_text = String::new();
                        
                        // Parse SSE format: data: {"candidates": [{"content": {"parts": [{"text": "..."}]}}]}
                        for line in text.lines() {
                            if line.starts_with("data: ") {
                                let json_str = &line[6..];
                                if let Ok(json) = serde_json::from_str::<serde_json::Value>(json_str) {
                                    if let Some(part_text) = json["candidates"][0]["content"]["parts"][0]["text"].as_str() {
                                        combined_text.push_str(part_text);
                                    }
                                }
                            }
                        }
                        combined_text
                    })
                });
                Box::pin(stream)
            }
            Err(e) => {
                Box::pin(futures::stream::once(async move { Err(e) }))
            }
        }
    }

    pub async fn detect_rep_agreement(&self, user_message: &str, ai_response: &str) -> Option<(String, String)> {
        if user_message.len() > 100 { return None; }

        let prompt = format!(
            "Analyze if the user's message: '{}' is an agreement to a specific small mental health action or 'rep' suggested by the AI in its previous response: '{}'.\n\nIf yes, return a JSON object with 'title' and 'reason'. If no, return {{}}.",
            user_message, ai_response
        );

        let body = json!({
            "contents": [{ "parts": [{ "text": prompt }] }],
            "generationConfig": { "response_mime_type": "application/json" }
        });

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
            self.api_key
        );

        if let Ok(resp) = self.client.post(url).json(&body).send().await {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                if let Some(text) = json["candidates"][0]["content"]["parts"][0]["text"].as_str() {
                    if let Ok(data) = serde_json::from_str::<serde_json::Value>(text) {
                        if let (Some(title), Some(reason)) = (data["title"].as_str(), data["reason"].as_str()) {
                            return Some((title.to_string(), reason.to_string()));
                        }
                    }
                }
            }
        }
        None
    }
}
