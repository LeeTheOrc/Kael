use std::path::PathBuf;

pub struct LlamaEngine;

impl LlamaEngine {
    pub fn new() -> Self {
        Self
    }
    
    pub fn get_models_dir() -> PathBuf {
        if let Some(proj_dirs) = directories::ProjectDirs::from("com", "kaelos", "Kael") {
            proj_dirs.data_dir().join("modals")
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
        false
    }
}

pub fn ensure_models_dir() {
    let dir = LlamaEngine::get_models_dir();
    if !dir.exists() {
        std::fs::create_dir_all(dir).ok();
    }
}
