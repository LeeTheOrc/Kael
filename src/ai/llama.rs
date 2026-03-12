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

    pub fn get_ai_model_path(ai_type: &str) -> PathBuf {
        let modals_dir = Self::get_models_dir();
        let model_file = match ai_type {
            "director" => "director.gguf",
            "programmer" => "programmer.gguf",
            "vision" => "vision.gguf",
            _ => "model.gguf",
        };
        modals_dir.join(ai_type).join(model_file)
    }

    pub fn load_ai_model(&mut self, ai_type: &str) -> Result<(), String> {
        let path = Self::get_ai_model_path(ai_type);
        if !path.exists() {
            return Err(format!("Model not found: {:?}", path));
        }
        self.load_model(path.to_str().unwrap_or(""))
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
