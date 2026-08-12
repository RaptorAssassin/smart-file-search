use std::time::Duration;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::services::ai::prompts::PROMPTS;

#[derive(Debug, Clone)]
pub struct OllamaClient {
    pub base_url: String,
    pub llm_model: String,
    pub embed_model: String,
    pub http: reqwest::Client,
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
}

impl OllamaClient {
    /// Builds a client pointed at an Ollama server.
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            llm_model: "gemma3:4b".to_string(),
            embed_model: "nomic-embed-text".to_string(),
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(600))
                .build()
                .expect("Failed to build HTTP client"),
        }
    }

    /// Sends a JSON body to an Ollama endpoint and deserializes the response, surfacing server errors.
    async fn post_json<T, R>(&self, path: &str, body: &T) -> Result<R, String>
    where
        T: Serialize,
        R: DeserializeOwned,
    {
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

    /// Asks the LLM for a short list of keywords for a piece of text.
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

        serde_json::from_str(&chat_response.message.content)
            .map_err(|e| format!("Failed to parse keyword JSON: {}", e))
    }

    /// Asks the LLM for a one-line summary of a piece of text.
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

        Ok(chat_response.message.content)
    }

    /// Asks the embedding model for a vector representation of the text.
    pub async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, String> {
        let body = EmbeddingRequest {
            model: self.embed_model.clone(),
            input: text.to_string(),
        };

        let embedding_response: EmbeddingResponse = self.post_json("/api/embed", &body).await?;

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

    /// Asks a vision model for a vector representation of an image.
    pub async fn generate_embedding_from_image(
        &self,
        base64_image: &str,
    ) -> Result<Vec<f32>, String> {
        let caption = self.generate_image_caption(base64_image).await?;
        self.generate_embedding(&caption).await
    }

    /// Asks the vision-capable model to describe an image in one short caption.
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

        if generate_response.response.trim().is_empty() {
            return Err("Vision model returned an empty caption".to_string());
        }

        Ok(generate_response.response)
    }
}
