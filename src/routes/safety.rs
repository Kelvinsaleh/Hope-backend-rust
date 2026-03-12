use axum::{
    http::StatusCode,
    response::IntoResponse,
    Json,
};

pub async fn get_safety_resources() -> impl IntoResponse {
    let resources = serde_json::json!({
        "success": true,
        "emergency": {
            "title": "Immediate Help",
            "number": "988",
            "description": "Crisis Lifeline (Available 24/7)"
        },
        "resources": [
            { "name": "Crisis Text Line", "contact": "Text HOME to 741741" },
            { "name": "The Trevor Project", "contact": "1-866-488-7386" },
            { "name": "NAMI HelpLine", "contact": "1-800-950-NAMI" }
        ]
    });

    (StatusCode::OK, Json(resources))
}
