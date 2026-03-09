use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use anyhow::Result;
use directories::ProjectDirs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub models: ModelConfig,
    pub chat: ChatConfig,
    pub vault: VaultConfig,
    pub api: ApiConfig,
    pub profiles: ProfileConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub director_model: String,
    pub programmer_model: String,
    pub vision_model: String,
    #[serde(default = "default_modals_dir")]
    pub model_dir: PathBuf,
}

fn default_modals_dir() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("com", "kaelos", "Kael") {
        proj_dirs.data_dir().join("modals")
    } else {
        PathBuf::from("modals")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatConfig {
    pub max_tokens: u32,
    pub temperature: f32,
    pub system_prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultConfig {
    #[serde(default = "default_vault_dir")]
    pub vault_dir: PathBuf,
    pub rag_enabled: bool,
    pub lora_enabled: bool,
}

fn default_vault_dir() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("com", "kaelos", "Kael") {
        proj_dirs.data_dir().join(".vault")
    } else {
        PathBuf::from(".vault")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub ollama_url: String,
    pub use_local: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileConfig {
    #[serde(default = "default_profiles_dir")]
    pub profiles_dir: PathBuf,
    pub active_profile: String,
}

fn default_profiles_dir() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("com", "kaelos", "Kael") {
        proj_dirs.data_dir().join(".profiles")
    } else {
        PathBuf::from(".profiles")
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            models: ModelConfig {
                director_model: "dolphin3.0-mistral-7b-q4".to_string(),
                programmer_model: "dolphin3.0-coder-7b-q4".to_string(),
                vision_model: "llava-7b-q4".to_string(),
                model_dir: default_modals_dir(),
            },
            chat: ChatConfig {
                max_tokens: 2048,
                temperature: 0.7,
                system_prompt: "You are Kael, a helpful AI assistant. You are direct, concise, and practical.".to_string(),
            },
            vault: VaultConfig {
                vault_dir: default_vault_dir(),
                rag_enabled: true,
                lora_enabled: true,
            },
            api: ApiConfig {
                ollama_url: "http://localhost:11434".to_string(),
                use_local: true,
            },
            profiles: ProfileConfig {
                profiles_dir: default_profiles_dir(),
                active_profile: "default".to_string(),
            },
        }
    }
}

impl Config {
    pub fn load(config_dir: &std::path::Path) -> Result<Self> {
        let config_path = config_dir.join("config.toml");
        
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            let config = Config::default();
            let content = toml::to_string_pretty(&config)?;
            std::fs::write(&config_path, content)?;
            Ok(config)
        }
    }
}
