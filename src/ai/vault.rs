use crate::ai::database::{Database, DatabaseStats, LoraConfig, RagDocument};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub struct Vault {
    pub database: Database,
    pub vault_path: PathBuf,
}

impl Vault {
    pub fn new() -> Result<Self, String> {
        let database = Database::new().map_err(|e| format!("Database error: {}", e))?;

        let vault_path = Self::get_vault_path();

        // Ensure vault directories exist
        let dirs = [
            vault_path.join("director/rag"),
            vault_path.join("director/lora"),
            vault_path.join("programmer/rag"),
            vault_path.join("programmer/lora"),
            vault_path.join("vision/rag"),
            vault_path.join("vision/lora"),
        ];

        for dir in &dirs {
            std::fs::create_dir_all(dir).map_err(|e| format!("Failed to create dir: {}", e))?;
        }

        Ok(Self {
            database,
            vault_path,
        })
    }

    fn get_vault_path() -> PathBuf {
        if let Some(proj_dirs) = directories::ProjectDirs::from("com", "kaelos", "Kael") {
            proj_dirs.data_dir().join(".vault")
        } else {
            PathBuf::from(".vault")
        }
    }

    // RAG Methods
    pub fn add_knowledge(
        &self,
        title: &str,
        content: &str,
        source: &str,
        ai_type: &str,
    ) -> Result<i64, String> {
        self.database
            .add_document(title, content, source, ai_type)
            .map_err(|e| format!("Failed to add document: {}", e))
    }

    pub fn search_knowledge(&self, query: &str, ai_type: &str) -> Result<Vec<RagDocument>, String> {
        self.database
            .search_documents(query, ai_type, 10)
            .map_err(|e| format!("Search failed: {}", e))
    }

    pub fn get_knowledge_base(&self, ai_type: &str) -> Result<Vec<RagDocument>, String> {
        self.database
            .get_all_documents(ai_type)
            .map_err(|e| format!("Failed to get documents: {}", e))
    }

    pub fn delete_knowledge(&self, id: i64) -> Result<(), String> {
        self.database
            .delete_document(id)
            .map_err(|e| format!("Failed to delete document: {}", e))
    }

    // LoRA Methods
    pub fn add_lora(
        &self,
        name: &str,
        ai_type: &str,
        description: Option<&str>,
    ) -> Result<i64, String> {
        let model_path = self
            .vault_path
            .join(ai_type)
            .join("lora")
            .join(format!("{}.bin", name));

        self.database
            .add_lora(
                name,
                ai_type,
                description,
                Some(model_path.to_str().unwrap_or("")),
            )
            .map_err(|e| format!("Failed to add LoRA: {}", e))
    }

    pub fn get_loras(&self, ai_type: Option<&str>) -> Result<Vec<LoraConfig>, String> {
        self.database
            .get_loras(ai_type)
            .map_err(|e| format!("Failed to get LoRAs: {}", e))
    }

    pub fn enable_lora(&self, id: i64, enabled: bool) -> Result<(), String> {
        self.database
            .enable_lora(id, enabled)
            .map_err(|e| format!("Failed to update LoRA: {}", e))
    }

    pub fn delete_lora(&self, id: i64) -> Result<(), String> {
        self.database
            .delete_lora(id)
            .map_err(|e| format!("Failed to delete LoRA: {}", e))
    }

    // Chat History
    pub fn save_message(
        &self,
        role: &str,
        content: &str,
        ai_mode: &str,
        request_type: Option<&str>,
    ) -> Result<i64, String> {
        self.database
            .add_message(role, content, ai_mode, request_type)
            .map_err(|e| format!("Failed to save message: {}", e))
    }

    pub fn get_chat_history(
        &self,
        ai_mode: &str,
        limit: i64,
    ) -> Result<Vec<crate::ai::database::ChatHistoryEntry>, String> {
        self.database
            .get_chat_history(ai_mode, limit)
            .map_err(|e| format!("Failed to get chat history: {}", e))
    }

    pub fn clear_history(&self, ai_mode: Option<&str>) -> Result<(), String> {
        self.database
            .clear_chat_history(ai_mode)
            .map_err(|e| format!("Failed to clear history: {}", e))
    }

    // Stats
    pub fn get_stats(&self) -> Result<DatabaseStats, String> {
        self.database
            .get_stats()
            .map_err(|e| format!("Failed to get stats: {}", e))
    }

    // Import knowledge from file/directory
    pub fn import_knowledge(&self, path: &str, ai_type: &str) -> Result<i64, String> {
        let path_obj = std::path::Path::new(path);

        if !path_obj.exists() {
            return Err(format!("Path does not exist: {}", path));
        }

        let content = if path_obj.is_file() {
            std::fs::read_to_string(path_obj).map_err(|e| format!("Failed to read file: {}", e))?
        } else {
            // Directory - read all files
            let mut all_content = String::new();
            if let Ok(entries) = std::fs::read_dir(path_obj) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let file_path = entry.path();
                    if file_path.is_file() {
                        if let Ok(file_content) = std::fs::read_to_string(&file_path) {
                            let file_name = file_path
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| "Unknown".to_string());
                            all_content.push_str(&format!("\n\n=== {} ===\n\n", file_name));
                            all_content.push_str(&file_content);
                        }
                    }
                }
            }
            all_content
        };

        let title = path_obj
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Imported".to_string());

        self.add_knowledge(&title, &content, path, ai_type)
    }
}
