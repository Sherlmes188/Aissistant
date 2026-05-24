use crate::config::AppConfig;
use anyhow::{anyhow, Context, Result};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage<'a>>,
    temperature: f32,
}

#[derive(Debug, Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Debug, Deserialize)]
struct ResponseMessage {
    content: String,
}

#[derive(Clone)]
pub struct ApiClient {
    client: Client,
}

impl ApiClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self { client }
    }

    pub fn ask(&self, config: &AppConfig, question: &str) -> Result<String> {
        ask_with_client(&self.client, config, question)
    }
}

fn ask_with_client(client: &Client, config: &AppConfig, question: &str) -> Result<String> {
    if config.api_key.trim().is_empty() {
        return Err(anyhow!("API key is empty"));
    }

    if question.trim().is_empty() {
        return Err(anyhow!("Question is empty"));
    }

    let url = format!(
        "{}/chat/completions",
        config.base_url.trim().trim_end_matches('/')
    );

    let body = ChatRequest {
        model: config.model.trim(),
        temperature: 0.2,
        messages: vec![
            ChatMessage {
                role: "system",
                content: config.system_prompt.trim(),
            },
            ChatMessage {
                role: "user",
                content: question.trim(),
            },
        ],
    };

    let response = client
        .post(url)
        .bearer_auth(config.api_key.trim())
        .json(&body)
        .send()
        .context("failed to send request")?;

    let status = response.status();
    let text = response.text().context("failed to read response")?;

    if !status.is_success() {
        return Err(anyhow!("API returned {status}: {text}"));
    }

    let parsed: ChatResponse =
        serde_json::from_str(&text).context("failed to parse chat response")?;

    parsed
        .choices
        .into_iter()
        .next()
        .map(|choice| choice.message.content)
        .filter(|content| !content.trim().is_empty())
        .ok_or_else(|| anyhow!("API returned an empty answer"))
}
