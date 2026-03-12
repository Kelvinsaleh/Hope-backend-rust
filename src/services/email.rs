use reqwest::Client;
use serde_json::json;
use crate::config::Config;
use std::time::SystemTime;
use chrono::{DateTime, Utc};

pub struct EmailService {
    client: Client,
    api_key: String,
    from_email: String,
}

impl EmailService {
    pub fn new() -> Self {
        let _config = Config::from_env();
        // Note: You'll need to add RESEND_API_KEY and FROM_EMAIL to your .env
        let api_key = std::env::var("RESEND_API_KEY").unwrap_or_default();
        let from_email = std::env::var("FROM_EMAIL").unwrap_or_else(|_| "onboarding@resend.dev".to_string());

        Self {
            client: Client::new(),
            api_key,
            from_email,
        }
    }

    pub async fn send_email(&self, to: &str, subject: &str, html: &str, text: &str) -> bool {
        if self.api_key.is_empty() {
            tracing::warn!("[DEV] No Resend API key. Email to={}: {}", to, text);
            return false;
        }

        let url = "https://api.resend.com/emails";
        let body = json!({
            "from": format!("Hope Therapy <{}>", self.from_email),
            "to": [to],
            "subject": subject,
            "html": html,
            "text": text,
        });

        match self.client.post(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await 
        {
            Ok(resp) => {
                if resp.status().is_success() {
                    tracing::info!("Email sent successfully to {}", to);
                    true
                } else {
                    let err = resp.text().await.unwrap_or_default();
                    tracing::error!("Failed to send email: {}", err);
                    false
                }
            }
            Err(e) => {
                tracing::error!("Network error sending email: {}", e);
                false
            }
        }
    }

    pub async fn send_verification_code(&self, email: &str, code: &str, name: &str) -> bool {
        let year = Utc::now().format("%Y").to_string();
        let html = format!(r#"<!DOCTYPE html><html><body style="font-family:sans-serif;max-width:600px;margin:0 auto">
<div style="background:#f9fafb;border-radius:12px;padding:40px;margin:20px 0">
  <div style="text-align:center"><h1 style="color:#6366f1">Hope Therapy</h1></div>
  <h2>Hello {name}!</h2>
  <p>Your email verification code:</p>
  <div style="background:white;border:2px solid #6366f1;border-radius:8px;padding:30px;text-align:center;margin:30px 0">
    <span style="font-size:42px;font-weight:bold;letter-spacing:8px;color:#6366f1;font-family:monospace">{code}</span>
    <p style="color:#666;font-size:14px">Expires in 10 minutes</p>
  </div>
  <div style="background:#fef3c7;border-left:4px solid #f59e0b;padding:15px;margin:20px 0">
    <strong>Security Note:</strong> Never share this code with anyone.
  </div>
  <p style="color:#999;font-size:12px;text-align:center">&copy; {year} Hope Therapy. All rights reserved.</p>
</div></body></html>"#);
        
        let text = format!("Hello {name}!\n\nVerification Code: {code}\n\nExpires in 10 minutes.");
        self.send_email(email, "Verify Your Hope Therapy Account", &html, &text).await
    }

    pub async fn send_welcome_email(&self, email: &str, name: &str) -> bool {
        let html = format!(r#"<!DOCTYPE html><html><body style="font-family:sans-serif;max-width:600px;margin:0 auto">
<div style="background:linear-gradient(135deg,#667eea,#764ba2);border-radius:12px;padding:40px;color:white">
  <h1 style="text-align:center">Welcome to Hope Therapy!</h1>
  <div style="background:white;color:#333;border-radius:8px;padding:30px;margin-top:20px">
    <h2>Hi {name}!</h2>
    <p>We are thrilled to have you join our community. Hope Therapy is your personal mental wellness companion.</p>
    <h3>What you can do now:</h3>
    <ul>
      <li>Start a Therapy Session with our AI companion</li>
      <li>Explore Guided Meditations</li>
      <li>Track Your Mood daily</li>
      <li>Journal Your Thoughts privately</li>
    </ul>
  </div>
</div></body></html>"#);
        
        let text = format!("Welcome to Hope Therapy, {name}!\n\nWe're glad to have you.");
        self.send_email(email, "Welcome to Hope Therapy!", &html, &text).await
    }
}
