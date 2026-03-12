use serde::Serialize;
use regex::Regex;
use std::collections::HashSet;

#[derive(Debug, Serialize, Default)]
pub struct ToneSignal {
    pub emotion: String,
    pub intensity: String,
    pub intent: String,
    pub clarity: String,
    pub signals: Vec<String>,
}

pub struct Analyzer;

impl Analyzer {
    pub fn analyze_tone(message: &str) -> ToneSignal {
        let lower = message.to_lowercase();
        let mut signals = Vec::new();
        
        // Count punctuation
        let exclamations = message.chars().filter(|&c| c == '!').count();
        let questions = message.chars().filter(|&c| c == '?').count();
        
        if exclamations >= 2 { signals.push("exclamations".to_string()); }
        if questions >= 2 { signals.push("many-questions".to_string()); }

        // Weighted Emotion Detection (Ported from C++)
        let mut emotion = "neutral".to_string();
        let mut max_weight = 0;

        let emotions = vec![
            ("sad", vec![("sad", 1), ("depressed", 2), ("hopeless", 3)]),
            ("anxious", vec![("anxious", 1), ("panic", 3), ("scared", 2)]),
            ("angry", vec![("angry", 1), ("furious", 3), ("rage", 3)]),
            ("happy", vec![("happy", 1), ("excited", 2), ("wonderful", 3)]),
        ];

        for (emo, kws) in emotions {
            for (kw, weight) in kws {
                if lower.contains(kw) {
                    // Check for Negation
                    if !Self::is_negated(&lower, kw) {
                        if weight > max_weight {
                            emotion = emo.to_string();
                            max_weight = weight;
                        }
                    }
                }
            }
        }

        ToneSignal {
            emotion,
            intensity: if max_weight >= 3 { "high".to_string() } else { "medium".to_string() },
            intent: "casual".to_string(), // Simplified for now
            clarity: "high".to_string(),
            signals,
        }
    }

    fn is_negated(text: &str, word: &str) -> bool {
        let negations = vec!["not", "never", "dont", "no"];
        // Simple check: see if a negation word appears shortly before the word
        // In a real production app, we'd use a more complex parser here
        for neg in negations {
            if text.contains(&format!("{} {}", neg, word)) {
                return true;
            }
        }
        false
    }

    pub fn extract_themes(content: &str) -> Vec<String> {
        let lower = content.to_lowercase();
        let mut themes = HashSet::new();

        if lower.contains("work") || lower.contains("job") { themes.insert("work".to_string()); }
        if lower.contains("relationship") || lower.contains("family") { themes.insert("relationships".to_string()); }
        if lower.contains("anxious") || lower.contains("worry") { themes.insert("anxiety".to_string()); }
        if lower.contains("sleep") { themes.insert("sleep".to_string()); }

        themes.into_iter().collect()
    }
}
