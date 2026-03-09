use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModel {
    pub name: String,
    pub size: u64,
    pub modified_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub stream: bool,
    pub options: ChatOptions,
}

#[derive(Debug, Serialize)]
pub struct ChatOptions {
    pub temperature: f32,
    pub num_predict: u32,
}

#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub message: ResponseMessage,
    pub done: bool,
}

#[derive(Debug, Deserialize)]
pub struct ResponseMessage {
    pub role: String,
    pub content: String,
}

pub struct OllamaClient {
    pub url: String,
    pub client: Client,
    pub default_model: String,
}

impl OllamaClient {
    pub fn new(url: String, default_model: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .expect("Failed to create HTTP client");
        
        Self {
            url,
            client,
            default_model,
        }
    }
    
    pub async fn chat(&self, messages: &[ChatMessage], model: Option<&str>) -> Result<String, String> {
        let model = model.unwrap_or(&self.default_model);
        
        let request = ChatRequest {
            model: model.to_string(),
            messages: messages.to_vec(),
            stream: false,
            options: ChatOptions {
                temperature: 0.7,
                num_predict: 2048,
            },
        };
        
        let response = self.client
            .post(format!("{}/api/chat", self.url))
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;
        
        if !response.status().is_success() {
            return Err(format!("Ollama returned status: {}", response.status()));
        }
        
        let chat_response: ChatResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;
        
        Ok(chat_response.message.content)
    }
    
    pub async fn generate(&self, prompt: &str, model: Option<&str>) -> Result<String, String> {
        let model = model.unwrap_or(&self.default_model);
        
        #[derive(Serialize)]
        struct GenerateRequest {
            model: String,
            prompt: String,
            stream: bool,
        }
        
        let request = GenerateRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            stream: false,
        };
        
        #[derive(Deserialize)]
        struct GenerateResponse {
            response: String,
        }
        
        let response = self.client
            .post(format!("{}/api/generate", self.url))
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;
        
        if !response.status().is_success() {
            return Err(format!("Ollama returned status: {}", response.status()));
        }
        
        let gen_response: GenerateResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;
        
        Ok(gen_response.response)
    }
    
    pub async fn is_available(&self) -> bool {
        match self.client
            .get(format!("{}/api/tags", self.url))
            .send()
            .await
        {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }
    
    pub async fn list_models(&self) -> Result<Vec<OllamaModel>, String> {
        let response = self.client
            .get(format!("{}/api/tags", self.url))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;
        
        #[derive(Deserialize)]
        struct ListResponse {
            models: Vec<OllamaModel>,
        }
        
        let list: ListResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;
        
        Ok(list.models)
    }
}
