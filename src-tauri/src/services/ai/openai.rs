use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::services::ai::prompts::PROMPTS;
use crate::services::usage::UsageCounters;

#[derive(Debug, Clone)]
pub struct OpenAiClient {
    pub endpoint: String,
    pub api_key: String,
    pub model: String,
    pub http: reqwest::Client,
    pub usage: Option<Arc<UsageCounters>>,
}

#[derive(Serialize, Debug, Clone)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatCompletionMessage>,
    stream: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ChatCompletionMessage {
    role: String,
    content: String,
}

#[derive(Deserialize, Debug, Clone)]
struct ChatCompletionResponse {
    choices: Vec<ChatCompletionChoice>,
    #[serde(default)]
    usage: Option<Usage>,
}

#[derive(Deserialize, Debug, Clone)]
struct ChatCompletionChoice {
    message: ChatCompletionMessage,
}

#[derive(Deserialize, Debug, Clone)]
struct Usage {
    #[serde(default)]
    prompt_tokens: u64,
    #[serde(default)]
    completion_tokens: u64,
}

impl OpenAiClient {
    pub fn new(endpoint: impl Into<String>, api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self::with_usage(endpoint, api_key, model, None)
    }

    pub fn with_usage(
        endpoint: impl Into<String>,
        api_key: impl Into<String>,
        model: impl Into<String>,
        usage: Option<Arc<UsageCounters>>,
    ) -> Self {
        Self {
            endpoint: endpoint.into(),
            api_key: api_key.into(),
            model: model.into(),
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(600))
                .build()
                .expect("Failed to build HTTP client"),
            usage,
        }
    }

    async fn chat(&self, system: &str, user: &str) -> Result<String, String> {
        let url = self.completions_url();

        if let Some(usage) = &self.usage {
            usage.record_request();
        }

        let body = ChatCompletionRequest {
            model: self.model.clone(),
            messages: vec![
                ChatCompletionMessage {
                    role: "system".to_string(),
                    content: system.to_string(),
                },
                ChatCompletionMessage {
                    role: "user".to_string(),
                    content: user.to_string(),
                },
            ],
            stream: false,
        };

        let res = self
            .http
            .post(url)
            .bearer_auth(self.api_key.clone())
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Failed to send request to /chat/completions: {e}"))?;

        if !res.status().is_success() {
            let status = res.status();
            let detail = res.text().await.unwrap_or_default();
            return Err(format!("Custom endpoint returned {status}: {detail}"));
        }

        let response: ChatCompletionResponse = res
            .json()
            .await
            .map_err(|e| format!("Failed to parse /chat/completions response: {e}"))?;

        if let Some(counter) = &self.usage {
            if let Some(response_usage) = &response.usage {
                counter.add_tokens(
                    response_usage
                        .prompt_tokens
                        .saturating_add(response_usage.completion_tokens),
                );
            }
        }

        response
            .choices
            .into_iter()
            .next()
            .map(|choice| choice.message.content)
            .ok_or_else(|| "Custom endpoint returned no choices".to_string())
    }

    pub async fn generate_keywords(&self, text: &str) -> Result<Vec<String>, String> {
        let content = self.chat(PROMPTS.keywords, text).await?;
        parse_keywords(&content)
    }

    pub async fn generate_summary(&self, text: &str) -> Result<String, String> {
        self.chat(PROMPTS.summary, text).await
    }

    fn completions_url(&self) -> String {
        if self.endpoint.ends_with("/chat/completions") {
            self.endpoint.clone()
        } else {
            format!("{}/chat/completions", self.endpoint.trim_end_matches('/'))
        }
    }
}

fn parse_keywords(content: &str) -> Result<Vec<String>, String> {
    let start = content.find('[');
    let end = content.rfind(']');
    let json = match (start, end) {
        (Some(s), Some(e)) if e > s => &content[s..=e],
        _ => return Err("No JSON array found in keyword reply".to_string()),
    };

    serde_json::from_str::<Vec<String>>(json)
        .map_err(|e| format!("Failed to parse keyword JSON: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn keywords_parse_from_chat_completions() {
        let mock_server = MockServer::start().await;
        let client = OpenAiClient::new(
            mock_server.uri(),
            "sk-test",
            "gpt-4o-mini",
        );

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "choices": [{
                    "message": { "role": "assistant", "content": "[\"rust\", \"ai\"]" }
                }],
                "usage": { "prompt_tokens": 12, "completion_tokens": 4 }
            })))
            .expect(1)
            .mount(&mock_server)
            .await;

        let result = client.generate_keywords("rust and ai").await;
        assert_eq!(result, Ok(vec!["rust".to_string(), "ai".to_string()]));
    }

    #[tokio::test]
    async fn summary_returns_raw_content() {
        let mock_server = MockServer::start().await;
        let client = OpenAiClient::new(mock_server.uri(), "sk-test", "gpt-4o-mini");

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "choices": [{
                    "message": { "role": "assistant", "content": "A short summary" }
                }]
            })))
            .expect(1)
            .mount(&mock_server)
            .await;

        let result = client.generate_summary("some text").await;
        assert_eq!(result, Ok("A short summary".to_string()));
    }

    #[tokio::test]
    async fn usage_records_requests_and_tokens() {
        let mock_server = MockServer::start().await;
        let usage = Arc::new(crate::services::usage::UsageCounters::default());
        let client = OpenAiClient::with_usage(
            mock_server.uri(),
            "sk-test",
            "gpt-4o-mini",
            Some(Arc::clone(&usage)),
        );

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "choices": [{
                    "message": { "role": "assistant", "content": "[\"rust\"]" }
                }],
                "usage": { "prompt_tokens": 10, "completion_tokens": 5 }
            })))
            .expect(1)
            .mount(&mock_server)
            .await;

        client.generate_keywords("rust").await.unwrap();

        let snap = usage.snapshot();
        assert_eq!(snap.requests, 1);
        assert_eq!(snap.tokens, 15);
    }

    #[tokio::test]
    async fn keywords_tolerate_prose_around_json() {
        let client = OpenAiClient::new("http://localhost:1", "sk-test", "gpt-4o-mini");
        let result = parse_keywords("Here are the keywords: [\"rust\", \"ai\"] Enjoy!");
        assert_eq!(result, Ok(vec!["rust".to_string(), "ai".to_string()]));
    }
}