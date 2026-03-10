use llama_gguf::engine::EngineConfig;
use llama_gguf::Engine;
use std::path::PathBuf;
use std::sync::Arc;

pub struct LlamaEngine {
    engine: Option<Arc<Engine>>,
    model_path: Option<PathBuf>,
}

impl LlamaEngine {
    pub fn new() -> Self {
        Self {
            engine: None,
            model_path: None,
        }
    }

    pub fn get_models_dir() -> PathBuf {
        // Look for modals folder in project root
        if let Ok(current_dir) = std::env::current_dir() {
            let modals_path = current_dir
                .parent()
                .map(|p| p.join("modals"))
                .unwrap_or_else(|| PathBuf::from("modals"));
            if modals_path.exists() {
                return modals_path;
            }
        }
        // Fallback to ~/.local/share/Kael/models
        if let Some(proj_dirs) = directories::ProjectDirs::from("com", "kaelos", "Kael") {
            proj_dirs.data_dir().join("models")
        } else {
            PathBuf::from("modals")
        }
    }

    pub fn list_available_models() -> Vec<String> {
        let dir = Self::get_models_dir();
        if !dir.exists() {
            return vec![];
        }

        std::fs::read_dir(&dir)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter(|e| {
                        let path = e.path();
                        path.extension().map(|ext| ext == "gguf").unwrap_or(false)
                    })
                    .filter_map(|e| e.file_name().into_string().ok())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn is_loaded(&self) -> bool {
        self.engine.is_some()
    }

    pub fn get_model_path(&self) -> Option<PathBuf> {
        self.model_path.clone()
    }

    pub fn load_model(&mut self, model_path: &str) -> Result<(), String> {
        let config = EngineConfig {
            model_path: model_path.to_string(),
            tokenizer_path: None,
            temperature: 0.7,
            top_k: 40,
            top_p: 0.95,
            repeat_penalty: 1.1,
            max_tokens: 2048,
            seed: None,
            use_gpu: false,
        };

        match Engine::load(config) {
            Ok(engine) => {
                tracing::info!("Loaded model: {}", model_path);
                self.engine = Some(Arc::new(engine));
                self.model_path = Some(PathBuf::from(model_path));
                Ok(())
            }
            Err(e) => Err(format!("Failed to load model: {}", e)),
        }
    }

    pub fn generate(&self, prompt: &str, max_tokens: Option<usize>) -> Result<String, String> {
        let engine = self
            .engine
            .as_ref()
            .ok_or_else(|| "No model loaded".to_string())?;

        let max = max_tokens.unwrap_or(512);
        engine
            .generate(prompt, max)
            .map_err(|e| format!("Generation failed: {}", e))
    }

    pub fn chat(&self, system_prompt: &str, user_message: &str) -> Result<String, String> {
        let engine = self
            .engine
            .as_ref()
            .ok_or_else(|| "No model loaded".to_string())?;

        let full_prompt = format!("{} User: {} Assistant:", system_prompt, user_message);
        engine
            .generate(&full_prompt, 512)
            .map_err(|e| format!("Chat failed: {}", e))
    }

    pub fn unload(&mut self) {
        self.engine = None;
        self.model_path = None;
    }
}

pub fn ensure_models_dir() {
    let dir = LlamaEngine::get_models_dir();
    if !dir.exists() {
        std::fs::create_dir_all(dir).ok();
    }
}
