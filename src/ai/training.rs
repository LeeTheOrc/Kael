use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
    pub total_items: i64,
    pub baked_items: i64,
    pub unbaked_items: i64,
    pub categories: Vec<String>,
    pub knowledge_growth_rate: f32,
}

pub struct TrainingPipeline {
    db_path: PathBuf,
}

impl TrainingPipeline {
    pub fn new() -> Self {
        let modals_dir = if let Ok(current_dir) = std::env::current_dir() {
            current_dir
                .parent()
                .map(|p| p.join("modals"))
                .unwrap_or_else(|| PathBuf::from("modals"))
        } else {
            PathBuf::from("modals")
        };

        let db_path = modals_dir.join("training.db");

        Self { db_path }
    }

    pub fn init(&self) -> Result<(), String> {
        let conn = Connection::open(&self.db_path).map_err(|e| e.to_string())?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS knowledge (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content TEXT NOT NULL,
                category TEXT NOT NULL,
                ai_type TEXT NOT NULL,
                confidence REAL DEFAULT 0.0,
                usage_count INTEGER DEFAULT 0,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                baked INTEGER DEFAULT 0
            )",
            [],
        )
        .map_err(|e| e.to_string())?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS training_sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                ai_type TEXT NOT NULL,
                started_at TEXT DEFAULT CURRENT_TIMESTAMP,
                completed_at TEXT,
                items_trained INTEGER DEFAULT 0,
                loss REAL DEFAULT 0.0,
                status TEXT DEFAULT 'pending'
            )",
            [],
        )
        .map_err(|e| e.to_string())?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS baked_models (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                ai_type TEXT NOT NULL,
                model_path TEXT NOT NULL,
                baked_at TEXT DEFAULT CURRENT_TIMESTAMP,
                knowledge_items INTEGER DEFAULT 0,
                version INTEGER DEFAULT 1
            )",
            [],
        )
        .map_err(|e| e.to_string())?;

        // Create indexes
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_knowledge_ai_type ON knowledge(ai_type)",
            [],
        )
        .map_err(|e| e.to_string())?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_knowledge_baked ON knowledge(baked)",
            [],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn add_knowledge(
        &self,
        content: &str,
        category: &str,
        ai_type: &str,
    ) -> Result<i64, String> {
        let conn = Connection::open(&self.db_path).map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT INTO knowledge (content, category, ai_type) VALUES (?1, ?2, ?3)",
            params![content, category, ai_type],
        )
        .map_err(|e| e.to_string())?;

        Ok(conn.last_insert_rowid())
    }

    pub fn get_knowledge(
        &self,
        ai_type: &str,
        unbaked_only: bool,
    ) -> Result<Vec<KnowledgeItem>, String> {
        let conn = Connection::open(&self.db_path).map_err(|e| e.to_string())?;

        let query = if unbaked_only {
            "SELECT id, content, category, ai_type, confidence, usage_count, created_at, baked 
             FROM knowledge WHERE ai_type = ?1 AND baked = 0 ORDER BY confidence DESC"
        } else {
            "SELECT id, content, category, ai_type, confidence, usage_count, created_at, baked 
             FROM knowledge WHERE ai_type = ?1 ORDER BY confidence DESC"
        };

        let mut stmt = conn.prepare(query).map_err(|e| e.to_string())?;

        let items = stmt
            .query_map(params![ai_type], |row| {
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

    pub fn update_confidence(&self, id: i64, correct: bool) -> Result<(), String> {
        let conn = Connection::open(&self.db_path).map_err(|e| e.to_string())?;

        let delta = if correct { 0.1 } else { -0.05 };

        conn.execute(
            "UPDATE knowledge SET confidence = MIN(1.0, MAX(0.0, confidence + ?1)), 
             usage_count = usage_count + 1 WHERE id = ?2",
            params![delta, id],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn mark_baked(&self, ids: &[i64]) -> Result<(), String> {
        let conn = Connection::open(&self.db_path).map_err(|e| e.to_string())?;

        for id in ids {
            conn.execute("UPDATE knowledge SET baked = 1 WHERE id = ?1", params![id])
                .map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    pub fn get_stats(&self, ai_type: &str) -> Result<TrainingStats, String> {
        let conn = Connection::open(&self.db_path).map_err(|e| e.to_string())?;

        let total: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM knowledge WHERE ai_type = ?1",
                params![ai_type],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let baked: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM knowledge WHERE ai_type = ?1 AND baked = 1",
                params![ai_type],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let categories: Vec<String> = {
            let mut stmt = conn
                .prepare("SELECT DISTINCT category FROM knowledge WHERE ai_type = ?1")
                .map_err(|e| e.to_string())?;

            let rows = stmt
                .query_map(params![ai_type], |row| row.get(0))
                .map_err(|e| e.to_string())?;

            rows.filter_map(|r| r.ok()).collect()
        };

        Ok(TrainingStats {
            total_items: total,
            baked_items: baked,
            unbaked_items: total - baked,
            categories,
            knowledge_growth_rate: 0.0,
        })
    }

    pub fn should_bake(&self, ai_type: &str, threshold: i64) -> Result<bool, String> {
        let stats = self.get_stats(ai_type)?;
        Ok(stats.unbaked_items >= threshold)
    }

    pub fn create_training_session(&self, ai_type: &str) -> Result<i64, String> {
        let conn = Connection::open(&self.db_path).map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT INTO training_sessions (ai_type, status) VALUES (?1, 'pending')",
            params![ai_type],
        )
        .map_err(|e| e.to_string())?;

        Ok(conn.last_insert_rowid())
    }

    pub fn complete_session(
        &self,
        session_id: i64,
        items_trained: i64,
        loss: f32,
    ) -> Result<(), String> {
        let conn = Connection::open(&self.db_path).map_err(|e| e.to_string())?;

        conn.execute(
            "UPDATE training_sessions SET status = 'completed', completed_at = CURRENT_TIMESTAMP,
             items_trained = ?1, loss = ?2 WHERE id = ?3",
            params![items_trained, loss, session_id],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn get_training_context(&self, ai_type: &str, max_tokens: usize) -> String {
        if let Ok(items) = self.get_knowledge(ai_type, true) {
            let mut context = String::from("## Knowledge Base\n\n");

            let mut current_tokens = 0;
            for item in items {
                let item_text = format!("- [{}] {}\n", item.category, item.content);
                if current_tokens + item_text.len() > max_tokens {
                    break;
                }
                context.push_str(&item_text);
                current_tokens += item_text.len();
            }

            context
        } else {
            String::new()
        }
    }
}

impl Default for TrainingPipeline {
    fn default() -> Self {
        Self::new()
    }
}
