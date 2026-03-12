use directories::ProjectDirs;
use std::path::PathBuf;
use tokio::fs;

#[derive(Clone)]
pub struct ModelDownloader {
    hf_token: Option<String>,
}

impl ModelDownloader {
    pub fn new() -> Self {
        let hf_token = std::env::var("HF_TOKEN").ok();
        Self { hf_token }
    }

    pub fn get_modals_dir() -> PathBuf {
        // Use current working directory as project root
        if let Ok(cwd) = std::env::current_dir() {
            // First check current dir
            let modals = cwd.join("modals");
            if modals.exists() {
                return modals;
            }
            // Check parent (if running from apps/kael)
            if let Some(parent) = cwd.parent() {
                let modals = parent.join("modals");
                if modals.exists() {
                    return modals;
                }
            }
        }
        
        // Fallback
        PathBuf::from("modals")
    }

    pub fn get_model_path(ai_type: &str) -> PathBuf {
        let modals_dir = Self::get_modals_dir();
        let model_file = match ai_type {
            "director" => "director.gguf",
            "programmer" => "programmer.gguf", 
            "vision" => "vision.gguf",
            _ => "model.gguf",
        };
        modals_dir.join(ai_type).join(model_file)
    }

    pub fn model_exists(ai_type: &str) -> bool {
        Self::get_model_path(ai_type).exists()
    }

    pub fn list_downloaded_models() -> Vec<(String, PathBuf)> {
        let mut models = Vec::new();
        let modals_dir = Self::get_modals_dir();
        
        for ai_type in ["director", "programmer", "vision"] {
            let path = Self::get_model_path(ai_type);
            if path.exists() {
                models.push((ai_type.to_string(), path));
            }
        }
        models
    }

    pub async fn download_model(&self, ai_type: &str) -> Result<PathBuf, String> {
        let model_path = Self::get_model_path(ai_type);
        
        if model_path.exists() {
            return Ok(model_path);
        }

        // Create directory
        if let Some(parent) = model_path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| e.to_string())?;
        }

        // Public vision models - using RaincloudAi which doesn't require auth
        let (repo_id, filename) = match ai_type {
            "director" => ("TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF", "tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf"),
            "programmer" => ("TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF", "tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf"),
            "vision" => ("RaincloudAi/llava-llama-3-8b-v1_1-Q4_K_M-GGUF", "llava-llama-3-8b-v1_1.Q4_K_M.gguf"),
            _ => return Err("Unknown AI type".to_string()),
        };

        let url = format!(
            "https://huggingface.co/{}/resolve/main/{}",
            repo_id, filename
        );

        println!("Downloading {} model from HuggingFace...", ai_type);
        println!("URL: {}", url);

        // Download with reqwest
        let client = reqwest::Client::new();
        let mut response = client.get(&url)
            .header("User-Agent", "Kael/1.0")
            .send()
            .await
            .map_err(|e| format!("Failed to download: {}", e))?;

        if !response.status().is_success() {
            // Try alternative model
            let alt_url = match ai_type {
                "director" | "programmer" => {
                    "https://huggingface.co/TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF/resolve/main/tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf"
                }
                "vision" => {
                    "https://huggingface.co/RaincloudAi/llava-llama-3-8b-v1_1-Q4_K_M-GGUF/resolve/main/llava-llama-3-8b-v1_1.Q4_K_M.gguf"
                }
                _ => return Err("Unknown AI type".to_string()),
            };
            
            println!("Trying alternative model...");
            response = client.get(alt_url)
                .header("User-Agent", "Kael/1.0")
                .send()
                .await
                .map_err(|e| format!("Failed to download alternative: {}", e))?;
        }

        // Download to file - read all bytes at once (for smaller models)
        let bytes = response.bytes().await.map_err(|e| format!("Download error: {}", e))?;
        
        fs::write(&model_path, &bytes).await.map_err(|e| format!("Failed to write file: {}", e))?;
        
        println!("Download complete! {} bytes", bytes.len());
        Ok(model_path)
    }

    pub async fn download_all_models(&self) -> Result<Vec<(String, PathBuf)>, String> {
        let mut downloaded = Vec::new();
        
        for ai_type in ["director", "programmer", "vision"] {
            if !Self::model_exists(ai_type) {
                match self.download_model(ai_type).await {
                    Ok(path) => downloaded.push((ai_type.to_string(), path)),
                    Err(e) => println!("Failed to download {}: {}", ai_type, e),
                }
            }
        }
        
        Ok(downloaded)
    }
}

pub fn ensure_modals_dir() {
    let dir = ModelDownloader::get_modals_dir();
    if !dir.exists() {
        std::fs::create_dir_all(&dir).ok();
    }
    
    // Create subdirectories for models
    for subdir in ["director", "programmer", "vision"] {
        let path = dir.join(subdir);
        if !path.exists() {
            std::fs::create_dir_all(&path).ok();
        }
    }
    
    // Also create .vault subdirectories for training databases
    let vault_dir = if let Ok(current_dir) = std::env::current_dir() {
        current_dir.parent()
            .map(|p| p.join(".vault"))
            .unwrap_or_else(|| PathBuf::from(".vault"))
    } else {
        PathBuf::from(".vault")
    };
    
    for subdir in ["director", "programmer", "vision"] {
        let path = vault_dir.join(subdir);
        if !path.exists() {
            std::fs::create_dir_all(&path).ok();
        }
    }
}
