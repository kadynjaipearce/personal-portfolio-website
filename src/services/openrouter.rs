use crate::config::CONFIG;
use crate::error::AppError;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

static SYSTEM_PROMPT: Lazy<String> = Lazy::new(|| {
    let path = Path::new("system-prompt.md");
    fs::read_to_string(path).unwrap_or_else(|_| {
        "You are a writing assistant for a personal engineering blog. Rewrite the given markdown to be clearer and better structured, but keep the original technical meaning and voice.".to_string()
    })
});

static FALLBACK_MODEL: &str = "openai/gpt-4o-mini";

#[derive(Serialize)]
struct OpenRouterMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct OpenRouterRequest {
    model: String,
    messages: Vec<OpenRouterMessage>,
}

#[derive(Deserialize)]
struct OpenRouterChoice {
    message: OpenRouterMessageContent,
}

#[derive(Deserialize)]
struct OpenRouterMessageContent {
    content: String,
}

#[derive(Deserialize)]
struct OpenRouterResponse {
    choices: Vec<OpenRouterChoice>,
}

pub struct OpenRouterService;

impl OpenRouterService {
    pub async fn rewrite_markdown(prompt: &str) -> Result<String, AppError> {
        let api_key = std::env::var("OPENROUTER_API_KEY")
            .map_err(|_| AppError::InternalError("OPENROUTER_API_KEY not set".into()))?;

        let primary_model =
            std::env::var("OPENROUTER_MODEL").unwrap_or_else(|_| "qwen/qwen-3.5-32b-a3b".into());

        let client = reqwest::Client::new();
        let make_body = |model: &str| OpenRouterRequest {
            model: model.to_string(),
            messages: vec![
                OpenRouterMessage {
                    role: "system".into(),
                    content: SYSTEM_PROMPT.clone(),
                },
                OpenRouterMessage {
                    role: "user".into(),
                    content: prompt.to_string(),
                },
            ],
        };

        let mut last_err: Option<AppError> = None;
        for model in [&primary_model[..], FALLBACK_MODEL] {
            let body = make_body(model);
            let res = client
                .post("https://openrouter.ai/api/v1/chat/completions")
                .bearer_auth(&api_key)
                .header("HTTP-Referer", &CONFIG.site_base_url)
                .header("X-Title", "Kadyn Pearce Portfolio")
                .json(&body)
                .send()
                .await;

            let res = match res {
                Ok(r) => r,
                Err(e) => {
                    last_err = Some(AppError::InternalError(format!(
                        "OpenRouter request failed for model {}: {}",
                        model, e
                    )));
                    continue;
                }
            };

            if !res.status().is_success() {
                last_err = Some(AppError::InternalError(format!(
                    "OpenRouter returned status {} for model {}",
                    res.status(),
                    model
                )));
                continue;
            }

            let data: OpenRouterResponse = res
                .json()
                .await
                .map_err(|e| AppError::InternalError(format!("OpenRouter response parse failed: {}", e)))?;

            let content = data
                .choices
                .into_iter()
                .next()
                .map(|c| c.message.content)
                .ok_or_else(|| AppError::InternalError("OpenRouter returned no choices".into()))?;

            return Ok(content);
        }

        Err(last_err.unwrap_or_else(|| {
            AppError::InternalError("OpenRouter request failed for all models".into())
        }))
    }
}

