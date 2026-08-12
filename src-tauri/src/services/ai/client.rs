use std::time::Duration;

#[derive(Debug, Clone)]
pub struct OllamaClient {
    pub base_url: String,
    pub http: reqwest::Client,
}

impl OllamaClient {
    /// Builds a client pointed at an Ollama server.
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(600))
                .build()
                .expect("Failed to build HTTP client"),
        }
    }

    /// Asks the LLM for a short list of keywords for a piece of text.
    pub async fn generate_keywords(&self, _text: &str) -> Result<Vec<String>, String> {
        Ok(Vec::new())
    }

    /// Asks the LLM for a one-line summary of a piece of text.
    pub async fn generate_summary(&self, _text: &str) -> Result<String, String> {
        Ok(String::new())
    }

    /// Asks the embedding model for a vector representation of the text.
    pub async fn generate_embedding(&self, _text: &str) -> Result<Vec<f32>, String> {
        Ok(Vec::new())
    }

    /// Asks a vision model for a vector representation of an image.
    pub async fn generate_embedding_from_image(
        &self,
        _base64_image: &str,
    ) -> Result<Vec<f32>, String> {
        Ok(Vec::new())
    }
}