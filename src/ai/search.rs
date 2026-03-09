use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

pub struct SearchEngine {
    pub use_ollama: bool,
    pub ollama_url: String,
}

impl SearchEngine {
    pub fn new(ollama_url: String) -> Self {
        Self {
            use_ollama: true,
            ollama_url,
        }
    }
    
    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>, String> {
        // For now, use a simple approach - ask Ollama for information
        // In production, you might want to add a real search API (Google, Bing, etc.)
        
        let prompt = format!(
            "Provide a helpful, accurate answer to this question: {}\n\nIf you don't know, say so honestly.",
            query
        );
        
        // This will be replaced with actual Ollama call
        Ok(vec![SearchResult {
            title: "AI Generated Answer".to_string(),
            url: "local://ollama".to_string(),
            snippet: format!("Answer from local AI for: {}", query),
        }])
    }
    
    pub fn extract_search_terms(message: &str) -> Option<String> {
        let msg_lower = message.to_lowercase();
        
        // Common search patterns
        let patterns = [
            "search for ",
            "look up ",
            "find information about ",
            "what is ",
            "who is ",
            "how to ",
            "when did ",
            "where is ",
            "why is ",
        ];
        
        for pattern in patterns {
            if let Some(idx) = msg_lower.find(pattern) {
                let start = idx + pattern.len();
                // Clean up the search query
                let query = message[start..].trim();
                if !query.is_empty() && query.len() > 2 {
                    return Some(query.to_string());
                }
            }
        }
        
        None
    }
}
