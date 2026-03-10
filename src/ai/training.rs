use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeItem {
    pub id: i64,
    pub content: String,
    pub category: String,
    pub ai_type: String,
    pub confidence: f32,
    pub usage_count: i64,
    pub created_at: String,
    pub baked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingStats {
    pub ai_type: String,
    pub total_items: i64,
    pub baked_items: i64,
    pub unbaked_items: i64,
    pub categories: Vec<String>,
    pub knowledge_growth_rate: f32,
    pub sessions_trained: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoraConfig {
    pub id: i64,
    pub ai_type: String,
    pub name: String,
    pub rank: i32,
    pub alpha: f32,
    pub enabled: bool,
    pub created_at: String,
}

pub struct AiTrainingSystem {
    ai_type: String,
    db_path: PathBuf,
    lock: Mutex<()>,
}

impl AiTrainingSystem {
    pub fn new(ai_type: &str) -> Self {
        let modals_dir = if let Ok(current_dir) = std::env::current_dir() {
            current_dir
                .parent()
                .map(|p| p.join("modals"))
                .unwrap_or_else(|| PathBuf::from("modals"))
        } else {
            PathBuf::from("modals")
        };

        let ai_dir = modals_dir.join(ai_type);
        let db_path = ai_dir.join("training.db");

        Self {
            ai_type: ai_type.to_string(),
            db_path,
            lock: Mutex::new(()),
        }
    }

    pub fn init(&self) -> Result<(), String> {
        // Create AI-specific directory
        if let Some(parent) = self.db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let conn = Connection::open(&self.db_path).map_err(|e| e.to_string())?;

        // Knowledge base - what the AI has learned
        conn.execute(
            "CREATE TABLE IF NOT EXISTS knowledge (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content TEXT NOT NULL,
                category TEXT NOT NULL,
                confidence REAL DEFAULT 0.5,
                usage_count INTEGER DEFAULT 0,
                feedback_score INTEGER DEFAULT 0,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                baked INTEGER DEFAULT 0,
                source TEXT DEFAULT 'interaction'
            )",
            [],
        )
        .map_err(|e| e.to_string())?;

        // LoRA adapters for fine-tuning
        conn.execute(
            "CREATE TABLE IF NOT EXISTS loras (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                rank INTEGER DEFAULT 4,
                alpha REAL DEFAULT 1.0,
                enabled INTEGER DEFAULT 0,
                trained_at TEXT DEFAULT CURRENT_TIMESTAMP,
                epochs_trained INTEGER DEFAULT 0,
                loss REAL DEFAULT 0.0
            )",
            [],
        )
        .map_err(|e| e.to_string())?;

        // Training sessions log
        conn.execute(
            "CREATE TABLE IF NOT EXISTS training_sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                started_at TEXT DEFAULT CURRENT_TIMESTAMP,
                completed_at TEXT,
                items_trained INTEGER DEFAULT 0,
                loss REAL DEFAULT 0.0,
                status TEXT DEFAULT 'pending',
                notes TEXT
            )",
            [],
        )
        .map_err(|e| e.to_string())?;

        // User interactions for learning
        conn.execute(
            "CREATE TABLE IF NOT EXISTS interactions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_message TEXT NOT NULL,
                ai_response TEXT NOT NULL,
                feedback INTEGER DEFAULT 0,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )
        .map_err(|e| e.to_string())?;

        // Baked knowledge snapshots
        conn.execute(
            "CREATE TABLE IF NOT EXISTS baked_knowledge (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                version INTEGER DEFAULT 1,
                items_count INTEGER DEFAULT 0,
                baked_at TEXT DEFAULT CURRENT_TIMESTAMP,
                model_hash TEXT,
                notes TEXT
            )",
            [],
        )
        .map_err(|e| e.to_string())?;

        // Create indexes
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_knowledge_category ON knowledge(category)",
            [],
        )
        .map_err(|e| e.to_string())?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_knowledge_baked ON knowledge(baked)",
            [],
        )
        .map_err(|e| e.to_string())?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_interactions_feedback ON interactions(feedback)",
            [],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn add_knowledge(
        &self,
        content: &str,
        category: &str,
        source: &str,
    ) -> Result<i64, String> {
        let _lock = self.lock.lock().map_err(|e| e.to_string())?;

        let conn = Connection::open(&self.db_path).map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT INTO knowledge (content, category, source) VALUES (?1, ?2, ?3)",
            params![content, category, source],
        )
        .map_err(|e| e.to_string())?;

        Ok(conn.last_insert_rowid())
    }

    pub fn record_interaction(&self, user_msg: &str, ai_response: &str) -> Result<i64, String> {
        let _lock = self.lock.lock().map_err(|e| e.to_string())?;

        let conn = Connection::open(&self.db_path).map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT INTO interactions (user_message, ai_response) VALUES (?1, ?2)",
            params![user_msg, ai_response],
        )
        .map_err(|e| e.to_string())?;

        Ok(conn.last_insert_rowid())
    }

    pub fn add_feedback(&self, interaction_id: i64, feedback: i32) -> Result<(), String> {
        let conn = Connection::open(&self.db_path).map_err(|e| e.to_string())?;

        conn.execute(
            "UPDATE interactions SET feedback = ?1 WHERE id = ?2",
            params![feedback, interaction_id],
        )
        .map_err(|e| e.to_string())?;

        // Also update knowledge confidence based on feedback
        let delta = if feedback > 0 { 0.1 } else { -0.05 };
        conn.execute(
            "UPDATE knowledge SET confidence = MIN(1.0, MAX(0.0, confidence + ?1)),
             usage_count = usage_count + 1 WHERE id IN (
                 SELECT id FROM knowledge ORDER BY confidence ASC LIMIT 10
             )",
            params![delta],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn get_knowledge(
        &self,
        category: Option<&str>,
        unbaked_only: bool,
    ) -> Result<Vec<KnowledgeItem>, String> {
        let conn = Connection::open(&self.db_path).map_err(|e| e.to_string())?;

        let query = match (category, unbaked_only) {
            (Some(cat), true) => {
                format!("SELECT id, content, category, '{}', confidence, usage_count, created_at, baked 
                         FROM knowledge WHERE category = '{}' AND baked = 0 ORDER BY confidence DESC", self.ai_type, cat)
            }
            (Some(cat), false) => {
                format!("SELECT id, content, category, '{}', confidence, usage_count, created_at, baked 
                         FROM knowledge WHERE category = '{}' ORDER BY confidence DESC", self.ai_type, cat)
            }
            (None, true) => {
                "SELECT id, content, category, ai_type, confidence, usage_count, created_at, baked 
                 FROM knowledge WHERE baked = 0 ORDER BY confidence DESC"
                    .to_string()
            }
            (None, false) => {
                "SELECT id, content, category, ai_type, confidence, usage_count, created_at, baked 
                 FROM knowledge ORDER BY confidence DESC"
                    .to_string()
            }
        };

        let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;

        let items = stmt
            .query_map([], |row| {
                Ok(KnowledgeItem {
                    id: row.get(0)?,
                    content: row.get(1)?,
                    category: row.get(2)?,
                    ai_type: row.get(3)?,
                    confidence: row.get(4)?,
                    usage_count: row.get(5)?,
                    created_at: row.get(6)?,
                    baked: row.get::<_, i32>(7)? != 0,
                })
            })
            .map_err(|e| e.to_string())?;

        items
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn get_context_for_prompt(&self, max_tokens: usize) -> String {
        if let Ok(items) = self.get_knowledge(None, true) {
            let mut context = format!("## {} Knowledge Base\n\n", self.ai_type);

            // Group by category
            let mut by_category: std::collections::HashMap<String, Vec<String>> =
                std::collections::HashMap::new();
            for item in items {
                by_category
                    .entry(item.category.clone())
                    .or_default()
                    .push(item.content);
            }

            for (category, contents) in by_category {
                context.push_str(&format!("### {}\n", category));
                for content in contents.iter().take(5) {
                    context.push_str(&format!("- {}\n", content));
                }
                context.push('\n');
            }

            // Truncate if too long
            if context.len() > max_tokens * 4 {
                context.truncate(max_tokens * 4);
                context.push_str("\n...[truncated]");
            }

            context
        } else {
            String::new()
        }
    }

    pub fn get_stats(&self) -> Result<TrainingStats, String> {
        let conn = Connection::open(&self.db_path).map_err(|e| e.to_string())?;

        let total: i64 = conn
            .query_row("SELECT COUNT(*) FROM knowledge", [], |row| row.get(0))
            .unwrap_or(0);

        let baked: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM knowledge WHERE baked = 1",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let sessions: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM training_sessions WHERE status = 'completed'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let categories: Vec<String> = {
            let mut stmt = conn
                .prepare("SELECT DISTINCT category FROM knowledge")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |row| row.get(0))
                .map_err(|e| e.to_string())?;
            rows.filter_map(|r| r.ok()).collect()
        };

        Ok(TrainingStats {
            ai_type: self.ai_type.clone(),
            total_items: total,
            baked_items: baked,
            unbaked_items: total - baked,
            categories,
            knowledge_growth_rate: 0.0,
            sessions_trained: sessions,
        })
    }

    pub fn should_bake(&self, threshold: i64) -> Result<bool, String> {
        let stats = self.get_stats()?;
        Ok(stats.unbaked_items >= threshold)
    }

    pub fn create_lora(&self, name: &str, rank: i32, alpha: f32) -> Result<i64, String> {
        let conn = Connection::open(&self.db_path).map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT INTO loras (name, rank, alpha) VALUES (?1, ?2, ?3)",
            params![name, rank, alpha],
        )
        .map_err(|e| e.to_string())?;

        Ok(conn.last_insert_rowid())
    }

    pub fn get_loras(&self) -> Result<Vec<LoraConfig>, String> {
        let conn = Connection::open(&self.db_path).map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare("SELECT id, ai_type, name, rank, alpha, enabled, created_at FROM loras")
            .map_err(|e| e.to_string())?;

        let loras = stmt
            .query_map([], |row| {
                Ok(LoraConfig {
                    id: row.get(0)?,
                    ai_type: self.ai_type.clone(),
                    name: row.get(2)?,
                    rank: row.get(3)?,
                    alpha: row.get(4)?,
                    enabled: row.get::<_, i32>(5)? != 0,
                    created_at: row.get(6)?,
                })
            })
            .map_err(|e| e.to_string())?;

        loras
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn enable_lora(&self, id: i64, enabled: bool) -> Result<(), String> {
        let conn = Connection::open(&self.db_path).map_err(|e| e.to_string())?;

        conn.execute(
            "UPDATE loras SET enabled = ?1 WHERE id = ?2",
            params![enabled as i32, id],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn mark_knowledge_baked(&self, ids: &[i64]) -> Result<(), String> {
        let _lock = self.lock.lock().map_err(|e| e.to_string())?;

        let conn = Connection::open(&self.db_path).map_err(|e| e.to_string())?;

        for id in ids {
            conn.execute("UPDATE knowledge SET baked = 1 WHERE id = ?1", params![id])
                .map_err(|e| e.to_string())?;
        }

        // Record baked snapshot
        conn.execute(
            "INSERT INTO baked_knowledge (items_count, notes) VALUES (?1, ?2)",
            params![ids.len(), format!("Baked {} items", ids.len())],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn get_interactions_for_training(
        &self,
        limit: i64,
    ) -> Result<Vec<(String, String, i32)>, String> {
        let conn = Connection::open(&self.db_path).map_err(|e| e.to_string())?;

        let mut stmt = conn.prepare(
            "SELECT user_message, ai_response, feedback FROM interactions ORDER BY id DESC LIMIT ?1"
        ).map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![limit], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i32>(2)?,
                ))
            })
            .map_err(|e| e.to_string())?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }
}

pub struct TrainingManager {
    director: AiTrainingSystem,
    programmer: AiTrainingSystem,
    vision: AiTrainingSystem,
}

impl TrainingManager {
    pub fn new() -> Self {
        let director = AiTrainingSystem::new("director");
        let programmer = AiTrainingSystem::new("programmer");
        let vision = AiTrainingSystem::new("vision");

        // Initialize all
        director.init().ok();
        programmer.init().ok();
        vision.init().ok();

        Self {
            director,
            programmer,
            vision,
        }
    }

    pub fn for_ai(&self, ai_type: &str) -> &AiTrainingSystem {
        match ai_type {
            "director" => &self.director,
            "programmer" => &self.programmer,
            "vision" => &self.vision,
            _ => &self.director,
        }
    }

    pub fn get_all_stats(&self) -> Vec<TrainingStats> {
        vec![
            self.director.get_stats().unwrap_or(TrainingStats {
                ai_type: "director".to_string(),
                total_items: 0,
                baked_items: 0,
                unbaked_items: 0,
                categories: vec![],
                knowledge_growth_rate: 0.0,
                sessions_trained: 0,
            }),
            self.programmer.get_stats().unwrap_or(TrainingStats {
                ai_type: "programmer".to_string(),
                total_items: 0,
                baked_items: 0,
                unbaked_items: 0,
                categories: vec![],
                knowledge_growth_rate: 0.0,
                sessions_trained: 0,
            }),
            self.vision.get_stats().unwrap_or(TrainingStats {
                ai_type: "vision".to_string(),
                total_items: 0,
                baked_items: 0,
                unbaked_items: 0,
                categories: vec![],
                knowledge_growth_rate: 0.0,
                sessions_trained: 0,
            }),
        ]
    }
}

impl Default for TrainingManager {
    fn default() -> Self {
        Self::new()
    }
}
