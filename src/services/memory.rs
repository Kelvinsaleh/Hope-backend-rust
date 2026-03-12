use mongodb::Database;
use bson::{doc, oid::ObjectId, DateTime};
use futures::StreamExt;
use std::sync::Arc;
use regex::Regex;
use lazy_static::lazy_static;

pub struct MemoryService;

struct FactPattern {
    regex: Regex,
    category: String,
    importance: i32,
    template: String,
}

lazy_static! {
    static ref PATTERNS: Vec<FactPattern> = vec![
        // Identity
        FactPattern { regex: Regex::new(r"(?i)my name is ([A-Za-z]{2,20})\b").unwrap(), category: "identity".to_string(), importance: 10, template: "Name: {1}".to_string() },
        FactPattern { regex: Regex::new(r"(?i)\bI(?:'m| am) (\d{1,2}) years? old\b").unwrap(), category: "identity".to_string(), importance: 8, template: "Age: {1}".to_string() },
        // Family
        FactPattern { regex: Regex::new(r"(?i)I have (\w+) (kid|child|son|daughter|children)\b").unwrap(), category: "family".to_string(), importance: 8, template: "Has {1} {2}".to_string() },
        FactPattern { regex: Regex::new(r"(?i)\bmy (wife|husband|partner|girlfriend|boyfriend|spouse)\b").unwrap(), category: "family".to_string(), importance: 7, template: "Has a {1}".to_string() },
        FactPattern { regex: Regex::new(r"(?i)\bI(?:'m| am) (married|single|divorced|widowed|engaged)\b").unwrap(), category: "family".to_string(), importance: 7, template: "Relationship: {1}".to_string() },
        FactPattern { regex: Regex::new(r"(?i)\bI(?:'m| am) (?:a )?(?:single |stay-at-home )?(mom|dad|father|mother|parent)\b").unwrap(), category: "family".to_string(), importance: 7, template: "Is a {1}".to_string() },
        // Work / study
        FactPattern { regex: Regex::new(r"(?i)I work (?:as|at|for) (.{3,40}?)[.!?,\n]").unwrap(), category: "work".to_string(), importance: 7, template: "Works {1}".to_string() },
        FactPattern { regex: Regex::new(r"(?i)my job is (.{3,40}?)[.!?,\n]").unwrap(), category: "work".to_string(), importance: 7, template: "Job: {1}".to_string() },
        FactPattern { regex: Regex::new(r"(?i)\bI(?:'m| am) (?:a )?(?:student|studying)\b").unwrap(), category: "work".to_string(), importance: 6, template: "Is a student".to_string() },
        FactPattern { regex: Regex::new(r"(?i)\bI(?:'m| am) a ([a-z]+ )?([a-z]+(?:er|or|ist|ian))\b").unwrap(), category: "work".to_string(), importance: 7, template: "Works as {1}{2}".to_string() },
        // Struggles / challenges
        FactPattern { regex: Regex::new(r"(?i)I(?:'ve| have) been (?:struggling|dealing) with (.{5,60}?)[.!?,\n]").unwrap(), category: "struggle".to_string(), importance: 9, template: "Struggling with: {1}".to_string() },
        FactPattern { regex: Regex::new(r"(?i)\bI(?:'m| am) dealing with (.{5,60}?)[.!?,\n]").unwrap(), category: "struggle".to_string(), importance: 8, template: "Dealing with: {1}".to_string() },
        FactPattern { regex: Regex::new(r"(?i)I(?:'ve| have) been diagnosed with (.{3,40}?)[.!?,\n]").unwrap(), category: "struggle".to_string(), importance: 9, template: "Diagnosed with: {1}".to_string() },
        FactPattern { regex: Regex::new(r"(?i)\bI(?:'m| am) going through (.{5,50}?)[.!?,\n]").unwrap(), category: "struggle".to_string(), importance: 7, template: "Going through: {1}".to_string() },
        // Goals
        FactPattern { regex: Regex::new(r"(?i)my goal is (?:to )?(.{5,60}?)[.!?,\n]").unwrap(), category: "goal".to_string(), importance: 7, template: "Goal: {1}".to_string() },
        FactPattern { regex: Regex::new(r"(?i)\bI want to (?:work on|improve|stop|start|become|get) (.{5,50}?)[.!?,\n]").unwrap(), category: "goal".to_string(), importance: 6, template: "Wants to: {1}".to_string() },
        FactPattern { regex: Regex::new(r"(?i)\bI(?:'m| am) trying to (.{5,50}?)[.!?,\n]").unwrap(), category: "goal".to_string(), importance: 6, template: "Trying to: {1}".to_string() },
        FactPattern { regex: Regex::new(r"(?i)\bI(?:'m| am) working on (.{5,50}?)[.!?,\n]").unwrap(), category: "goal".to_string(), importance: 6, template: "Working on: {1}".to_string() },
        // Location
        FactPattern { regex: Regex::new(r"(?i)\bI(?:'m| am) from ([A-Za-z][A-Za-z ]{2,25}?)[.!?,\n]").unwrap(), category: "location".to_string(), importance: 4, template: "From: {1}".to_string() },
        FactPattern { regex: Regex::new(r"(?i)\bI live in ([A-Za-z][A-Za-z ]{2,25}?)[.!?,\n]").unwrap(), category: "location".to_string(), importance: 4, template: "Lives in: {1}".to_string() },
    ];
}

const MULTI_FACT_CATEGORIES: &[&str] = &["goal", "struggle"];

impl MemoryService {
    pub async fn process_message_for_memory(db: &Database, user_id: &str, message: &str) {
        let oid = match ObjectId::parse_str(user_id) {
            Ok(id) => id,
            Err(_) => return,
        };

        let mut extracted_facts = Vec::new();

        for pattern in PATTERNS.iter() {
            if let Some(captures) = pattern.regex.captures(message) {
                let mut content = pattern.template.clone();
                for i in 1..captures.len() {
                    let val = captures.get(i).map_or("", |m| m.as_str()).trim();
                    content = content.replace(&format!("{{{}}}", i), val);
                }
                content = content.trim().trim_end_matches(|c| c == '.' || c == '!' || c == '?' || c == ',').to_string();
                
                if content.len() >= 5 && content.len() <= 100 {
                    extracted_facts.push(doc! {
                        "category": &pattern.category,
                        "content": content,
                        "importance": pattern.importance,
                    });
                }
            }
        }

        if extracted_facts.is_empty() {
            return;
        }

        let collection = db.collection::<bson::Document>("long_term_memories");
        
        // Get existing facts to deduplicate
        let filter = doc! { "userId": oid };
        let mut cursor = match collection.find(filter, None).await {
            Ok(c) => c,
            Err(_) => return,
        };

        let mut existing_memories = Vec::new();
        while let Some(Ok(doc)) = cursor.next().await {
            existing_memories.push(doc);
        }

        let now = DateTime::now();

        for fact in extracted_facts {
            let category = fact.get_str("category").unwrap_or("");
            let content = fact.get_str("content").unwrap_or("");
            let content_lc = content.to_lowercase();
            let importance = fact.get_i32("importance").unwrap_or(0);

            // Skip if near-duplicate exists
            let is_duplicate = existing_memories.iter().any(|m| {
                let m_content = m.get_str("content").unwrap_or("").to_lowercase();
                let sub_new = &content_lc[..std::cmp::min(30, content_lc.len())];
                let sub_old = &m_content[..std::cmp::min(30, m_content.len())];
                m_content.contains(sub_new) || content_lc.contains(sub_old)
            });

            if is_duplicate {
                continue;
            }

            let is_multi = MULTI_FACT_CATEGORIES.contains(&category);
            let existing_of_cat = existing_memories.iter().find(|m| m.get_str("category").unwrap_or("") == category);

            let mut updated = false;
            if !is_multi {
                if let Some(existing) = existing_of_cat {
                    // Singleton update
                    let existing_importance = existing.get_i32("importance").unwrap_or(0);
                    if importance > existing_importance {
                        if let Ok(id) = existing.get_object_id("_id") {
                            let update = doc! {
                                "$set": {
                                    "content": content,
                                    "importance": importance,
                                    "updatedAt": now,
                                }
                            };
                            let _ = collection.update_one(doc! { "_id": id }, update, None).await;
                        }
                    }
                    updated = true;
                }
            }

            if !updated {
                // Insert new (either multi-fact or first of its category)
                let new_memory = doc! {
                    "userId": oid,
                    "category": category,
                    "content": content,
                    "importance": importance,
                    "source": "auto",
                    "tags": [category],
                    "createdAt": now,
                    "updatedAt": now,
                };
                let _ = collection.insert_one(new_memory, None).await;
            }
        }
    }

    pub async fn get_context(db: &Database, user_id: &str) -> String {
        let collection = db.collection::<serde_json::Value>("long_term_memories");
        
        let oid = match ObjectId::parse_str(user_id) {
            Ok(id) => id,
            Err(_) => return "".to_string(),
        };

        let filter = doc! {
            "userId": oid
        };

        let mut cursor = collection.find(filter, None).await.unwrap();
        let mut context = Vec::new();

        while let Some(result) = cursor.next().await {
            if let Ok(doc) = result {
                if let Some(content) = doc.get("content").and_then(|c| c.as_str()) {
                    context.push(content.to_string());
                }
            }
        }

        if context.is_empty() {
            "".to_string()
        } else {
            format!("The following facts are known about the user:\n- {}", context.join("\n- "))
        }
    }
}
