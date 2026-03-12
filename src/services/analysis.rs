use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct ToneAnalysis {
    pub emotion: String,
    pub intent: String,
    pub clarity: String,
    pub intensity: String,
    pub signals: Vec<String>,
    #[serde(rename = "recommendedMode")]
    pub recommended_mode: String,
    pub guidance: String,
}

#[derive(Serialize, Clone)]
pub struct ResiliencePattern {
    pub trigger: String,
    pub factor: String,
    pub confidence: f32,
}

pub struct AnalysisService;

impl AnalysisService {
    pub fn analyze_user_tone(message: &str) -> ToneAnalysis {
        let lower = message.to_lowercase();
        let mut emotion = "neutral";
        
        // Basic keyword matching
        if lower.contains("sad") || lower.contains("down") || lower.contains("depressed") || lower.contains("hopeless") {
            emotion = "sad";
        } else if lower.contains("anxious") || lower.contains("anxiety") || lower.contains("worried") || lower.contains("panic") {
            emotion = "anxious";
        } else if lower.contains("angry") || lower.contains("mad") || lower.contains("furious") {
            emotion = "angry";
        } else if lower.contains("frustrated") || lower.contains("annoyed") || lower.contains("stuck") {
            emotion = "frustrated";
        } else if lower.contains("happy") || lower.contains("excited") || lower.contains("great") || lower.contains("proud") {
            emotion = "happy";
        } else if lower.contains("confused") || lower.contains("lost") || lower.contains("not sure") || lower.contains("idk") {
            emotion = "confused";
        }

        let (mode, guidance) = match emotion {
            "sad" => ("EMPATHY MODE", "Validate their feelings gently."),
            "anxious" => ("GROUNDING MODE", "Focus on calming and centering."),
            "angry" => ("DE-ESCALATION MODE", "Acknowledge frustration without judgement."),
            "happy" => ("CELEBRATION MODE", "Share in their joy."),
            "frustrated" => ("PROBLEM-SOLVING MODE", "Help break down the issue."),
            _ => ("FRIEND MODE", "Keep it warm and brief."),
        };

        ToneAnalysis {
            emotion: emotion.to_string(),
            intent: "casual".to_string(),
            clarity: "medium".to_string(),
            intensity: "low".to_string(),
            signals: vec![],
            recommended_mode: mode.to_string(),
            guidance: guidance.to_string(),
        }
    }

    pub fn extract_key_themes(content: &str) -> Vec<String> {
        let lc = content.to_lowercase();
        let mut themes = Vec::new();
        
        if lc.contains("work") || lc.contains("job") || lc.contains("career") { themes.push("work".to_string()); }
        if lc.contains("relationship") || lc.contains("partner") || lc.contains("family") { themes.push("relationships".to_string()); }
        if lc.contains("anx") || lc.contains("worry") || lc.contains("panic") { themes.push("anxiety".to_string()); }
        if lc.contains("sleep") || lc.contains("insomnia") { themes.push("sleep".to_string()); }
        if lc.contains("health") || lc.contains("exercise") { themes.push("health".to_string()); }
        
        themes.sort();
        themes.dedup();
        themes.truncate(6);
        themes
    }

    pub fn compute_emotional_state(_content: &str, mood: i32) -> String {
        if mood <= 2 { return "very low".to_string(); }
        if mood == 3 { return "low".to_string(); }
        if mood == 4 { return "neutral".to_string(); }
        if mood == 5 { return "good".to_string(); }
        "excellent".to_string()
    }

    pub fn detect_resilience_patterns(facts: &[String]) -> Vec<ResiliencePattern> {
        let mut patterns = Vec::new();
        for fact in facts {
            let lc = fact.to_lowercase();
            if lc.contains("walk") || lc.contains("exercise") || lc.contains("run") {
                patterns.push(ResiliencePattern {
                    trigger: "Movement".to_string(),
                    factor: "Physical reset breaks the emotional loop".to_string(),
                    confidence: 0.85,
                });
            }
            if lc.contains("journal") || lc.contains("write") || lc.contains("wrote") {
                patterns.push(ResiliencePattern {
                    trigger: "Reflection".to_string(),
                    factor: "Externalizing thoughts reduces cognitive load".to_string(),
                    confidence: 0.92,
                });
            }
            if lc.contains("breath") || lc.contains("meditat") {
                patterns.push(ResiliencePattern {
                    trigger: "Stillness".to_string(),
                    factor: "Parasympathetic activation calms the nervous system".to_string(),
                    confidence: 0.88,
                });
            }
        }
        patterns.dedup_by(|a, b| a.trigger == b.trigger);
        patterns
    }

    pub fn generate_resilience_hypothesis(facts: &[String], current_tone: &str) -> Option<String> {
        let all_facts = facts.join(" ").to_lowercase();
        
        // Dot-connecting logic
        if all_facts.contains("work") && current_tone == "anxious" {
            return Some("I'm noticing a pattern where your professional successes seem to trigger a fear of 'being too much' or losing your balance.".to_string());
        }
        if all_facts.contains("family") && current_tone == "sad" {
            return Some("It seems like your deep capacity for care often leaves you feeling drained when family dynamics shift unexpectedly.".to_string());
        }
        if all_facts.contains("single") && current_tone == "lonely" {
            return Some("Your independence is a strength, but it sometimes feels like a wall you built to protect yourself from the weight of being seen.".to_string());
        }
        
        None
    }

    pub fn generate_personalized_mantra(facts: &[String], current_mood: &str) -> String {
        let all_facts = facts.join(" ").to_lowercase();
        
        if current_mood == "sad" || current_mood == "stressed" {
            if all_facts.contains("work") {
                return "My worth is not defined by my productivity, but by the steadiness of my heart.".to_string();
            }
            return "I have handled hard things before, and I will handle this too.".to_string();
        }
        
        if current_mood == "happy" || current_mood == "calm" {
            return "I am the hero of my own story, and my progress is real.".to_string();
        }

        "I am finding my way, one step at a time.".to_string()
    }
}
