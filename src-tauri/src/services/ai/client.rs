use std::sync::Arc;
use std::time::Duration;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::services::ai::prompts::PROMPTS;
use crate::services::usage::UsageCounters;

#[derive(Debug, Clone)]
pub struct OllamaClient {
    pub base_url: String,
    pub llm_model: String,
    pub embed_model: String,
    pub http: reqwest::Client,
    pub usage: Option<Arc<UsageCounters>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ChatResponse {
    message: ChatMessage,
    done: bool,
    #[serde(default)]
    prompt_eval_count: Option<u64>,
    #[serde(default)]
    eval_count: Option<u64>,
}

#[derive(Serialize, Debug, Clone)]
pub struct EmbeddingRequest {
    pub model: String,
    pub input: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct EmbeddingResponse {
    pub embeddings: Vec<Vec<f32>>,
}

#[derive(Serialize, Debug, Clone)]
pub struct GenerateRequest {
    pub model: String,
    pub prompt: String,
    pub images: Vec<String>,
    pub stream: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GenerateResponse {
    response: String,
    done: bool,
    #[serde(default)]
    prompt_eval_count: Option<u64>,
    #[serde(default)]
    eval_count: Option<u64>,
}

impl OllamaClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self::with_usage(base_url, None)
    }

    pub fn with_usage(base_url: impl Into<String>, usage: Option<Arc<UsageCounters>>) -> Self {
        Self {
            base_url: base_url.into(),
            llm_model: "gemma3:4b".to_string(),
            embed_model: "nomic-embed-text".to_string(),
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(600))
                .build()
                .expect("Failed to build HTTP client"),
            usage,
        }
    }

    async fn post_json<T, R>(&self, path: &str, body: &T) -> Result<R, String>
    where
        T: Serialize,
        R: DeserializeOwned,
    {
        if let Some(usage) = &self.usage {
            usage.record_request();
        }
        let url = format!("{}{}", self.base_url, path);
        let res = self
            .http
            .post(url)
            .json(body)
            .send()
            .await
            .map_err(|e| format!("Failed to send request to {}: {}", path, e))?;

        if !res.status().is_success() {
            let status = res.status();
            let detail = res.text().await.unwrap_or_default();
            return Err(format!("Ollama returned {status} for {path}: {detail}"));
        }

        res.json::<R>()
            .await
            .map_err(|e| format!("Failed to parse response from {}: {}", path, e))
    }

    fn record_tokens(&self, prompt_eval_count: Option<u64>, eval_count: Option<u64>) {
        if let Some(usage) = &self.usage {
            usage.add_tokens(prompt_eval_count.unwrap_or(0) + eval_count.unwrap_or(0));
        }
    }

    pub async fn generate_keywords(&self, text: &str) -> Result<Vec<String>, String> {
        let body = ChatRequest {
            model: self.llm_model.clone(),
            messages: vec![
                ChatMessage {
                    role: Role::System,
                    content: PROMPTS.keywords.to_string(),
                },
                ChatMessage {
                    role: Role::User,
                    content: text.to_string(),
                },
            ],
            stream: false,
            format: Some("json".to_string()),
        };

        let chat_response: ChatResponse = self.post_json("/api/chat", &body).await?;

        if !chat_response.done {
            return Err("Keyword generation not completed".to_string());
        }

        self.record_tokens(chat_response.prompt_eval_count, chat_response.eval_count);

        parse_keywords(&chat_response.message.content)
    }

    pub async fn generate_summary(&self, text: &str) -> Result<String, String> {
        let body = ChatRequest {
            model: self.llm_model.clone(),
            messages: vec![
                ChatMessage {
                    role: Role::System,
                    content: PROMPTS.summary.to_string(),
                },
                ChatMessage {
                    role: Role::User,
                    content: text.to_string(),
                },
            ],
            stream: false,
            format: None,
        };

        let chat_response: ChatResponse = self.post_json("/api/chat", &body).await?;

        if !chat_response.done {
            return Err("Summary generation not completed".to_string());
        }

        self.record_tokens(chat_response.prompt_eval_count, chat_response.eval_count);

        Ok(chat_response.message.content)
    }

    pub async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, String> {
        let body = EmbeddingRequest {
            model: self.embed_model.clone(),
            input: text.to_string(),
        };

        let embedding_response: EmbeddingResponse = self.post_json("/api/embed", &body).await?;

        if let Some(usage) = &self.usage {
            usage.add_tokens((text.chars().count() / 4) as u64);
        }

        let embedding = embedding_response
            .embeddings
            .into_iter()
            .next()
            .ok_or_else(|| "Embedding response contained no vectors".to_string())?;

        if embedding.len() != 768 {
            return Err(format!(
                "Embedding had {} dimensions, expected 768",
                embedding.len()
            ));
        }

        Ok(embedding)
    }

    pub async fn generate_embedding_from_image(
        &self,
        base64_image: &str,
    ) -> Result<Vec<f32>, String> {
        let caption = self.generate_image_caption(base64_image).await?;
        self.generate_embedding(&caption).await
    }

    async fn generate_image_caption(&self, base64_image: &str) -> Result<String, String> {
        let body = GenerateRequest {
            model: self.llm_model.clone(),
            prompt: PROMPTS.image_caption.to_string(),
            images: vec![base64_image.to_string()],
            stream: false,
        };

        let generate_response: GenerateResponse = self.post_json("/api/generate", &body).await?;

        if !generate_response.done {
            return Err("Vision generation not completed".to_string());
        }

        self.record_tokens(generate_response.prompt_eval_count, generate_response.eval_count);

        if generate_response.response.trim().is_empty() {
            return Err("Vision model returned an empty caption".to_string());
        }

        Ok(generate_response.response)
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

    #[test]
    fn new_sets_default_models() {
        let client = OllamaClient::new("http://localhost:11434");
        assert_eq!(client.llm_model, "gemma3:4b");
        assert_eq!(client.embed_model, "nomic-embed-text");
        assert_eq!(client.base_url, "http://localhost:11434");
        assert!(client.usage.is_none());
    }

    #[tokio::test]
    async fn usage_records_requests_and_tokens() {
        let mock_server = MockServer::start().await;
        let usage = Arc::new(crate::services::usage::UsageCounters::default());
        let client = OllamaClient::with_usage(mock_server.uri(), Some(Arc::clone(&usage)));

        Mock::given(method("POST"))
            .and(path("/api/chat"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "message": {
                    "role": "assistant",
                    "content": "[\"rust\"]"
                },
                "done": true,
                "prompt_eval_count": 10,
                "eval_count": 5
            })))
            .expect(1)
            .mount(&mock_server)
            .await;

        client.generate_keywords("rust and ai").await.unwrap();

        let snap = usage.snapshot();
        assert_eq!(snap.requests, 1);
        assert_eq!(snap.tokens, 15);
    }

    #[tokio::test]
    async fn keywords_hit_chat_endpoint() {
        let mock_server = MockServer::start().await;
        let client = OllamaClient::new(mock_server.uri());

        Mock::given(method("POST"))
            .and(path("/api/chat"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "message": {
                    "role": "assistant",
                    "content": "[\"rust\", \"ai\"]"
                },
                "done": true
            })))
            .expect(1)
            .mount(&mock_server)
            .await;

        let result = client.generate_keywords("rust and ai").await;

        assert_eq!(result, Ok(vec!["rust".to_string(), "ai".to_string()]));
    }

    #[tokio::test]
    async fn summary_omits_format_field() {
        let mock_server = MockServer::start().await;
        let client = OllamaClient::new(mock_server.uri());

        Mock::given(method("POST"))
            .and(path("/api/chat"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "message": {
                    "role": "assistant",
                    "content": "A short summary"
                },
                "done": true
            })))
            .expect(1)
            .mount(&mock_server)
            .await;

        let result = client.generate_summary("some text").await;
        assert_eq!(result, Ok("A short summary".to_string()));

        let requests = mock_server.received_requests().await.unwrap();
        let body: serde_json::Value =
            serde_json::from_slice(&requests[0].body).expect("request body is valid JSON");
        assert!(body.get("format").is_none(), "summary must not send format");
    }

    #[tokio::test]
    async fn embedding_rejects_wrong_dimensions() {
        let mock_server = MockServer::start().await;
        let client = OllamaClient::new(mock_server.uri());

        Mock::given(method("POST"))
            .and(path("/api/embed"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "embeddings": [[0.1, 0.2, 0.3]]
            })))
            .expect(1)
            .mount(&mock_server)
            .await;

        let err = client.generate_embedding("hello").await.unwrap_err();
        assert!(err.contains("expected 768"), "unexpected error: {err}");
    }

    #[tokio::test]
    async fn keywords_rejects_incomplete_response() {
        let mock_server = MockServer::start().await;
        let client = OllamaClient::new(mock_server.uri());

        Mock::given(method("POST"))
            .and(path("/api/chat"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "message": {
                    "role": "assistant",
                    "content": "[]"
                },
                "done": false
            })))
            .expect(1)
            .mount(&mock_server)
            .await;

        let err = client.generate_keywords("hello").await.unwrap_err();
        assert_eq!(err, "Keyword generation not completed");
    }

    #[tokio::test]
    async fn keywords_tolerate_wrapper_object() {
        let mock_server = MockServer::start().await;
        let client = OllamaClient::new(mock_server.uri());

        Mock::given(method("POST"))
            .and(path("/api/chat"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "message": {
                    "role": "assistant",
                    "content": "{\"keywords\": [\"rust\", \"ai\"]}"
                },
                "done": true
            })))
            .mount(&mock_server)
            .await;

        let result = client.generate_keywords("rust and ai").await;
        assert_eq!(result, Ok(vec!["rust".to_string(), "ai".to_string()]));
    }
}
