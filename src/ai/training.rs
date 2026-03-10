use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeItem {
    pub id: i64,
    pub content: String,
    pub category: String,
    pub importance: i32, // 0=trivial, 1=important, 2=critical
    pub confidence: f32,
    pub usage_count: i64,
    pub created_at: String,
    pub status: String, // sql, rag, lora, baked
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingStats {
    pub ai_type: String,
    pub sql_items: i64,
    pub rag_items: i64,
    pub lora_items: i64,
    pub baked_items: i64,
    pub total_items: i64,
    pub importance_breakdown: std::collections::HashMap<String, i64>,
    pub should_promote_to_rag: bool,
    pub should_create_lora: bool,
    pub should_bake: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoraConfig {
    pub id: i64,
    pub name: String,
    pub rank: i32,
    pub alpha: f32,
    pub trained_on: i64,
    pub enabled: bool,
    pub created_at: String,
}

pub struct AiTrainingSystem {
    ai_type: String,
    sql_db_path: PathBuf,
    rag_path: PathBuf,
    lock: Mutex<()>,
}

impl AiTrainingSystem {
    pub fn new(ai_type: &str) -> Self {
        let vault_dir = if let Ok(current_dir) = std::env::current_dir() {
            current_dir
                .parent()
                .map(|p| p.join(".vault"))
                .unwrap_or_else(|| PathBuf::from(".vault"))
        } else {
            PathBuf::from(".vault")
        };

        let ai_dir = vault_dir.join(ai_type);
        let sql_db_path = ai_dir.join("training.db");
        let rag_path = ai_dir.join("rag");

        Self {
            ai_type: ai_type.to_string(),
            sql_db_path,
            rag_path,
            lock: Mutex::new(()),
        }
    }

    pub fn init(&self) -> Result<(), String> {
        // Create directories
        if let Some(parent) = self.sql_db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::create_dir_all(&self.rag_path).map_err(|e| e.to_string())?;

        let conn = Connection::open(&self.sql_db_path).map_err(|e| e.to_string())?;

        // SQL storage - all data starts here
        conn.execute(
            "CREATE TABLE IF NOT EXISTS knowledge (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content TEXT NOT NULL,
                category TEXT NOT NULL,
                importance INTEGER DEFAULT 0,
                confidence REAL DEFAULT 0.5,
                usage_count INTEGER DEFAULT 0,
                feedback_score INTEGER DEFAULT 0,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                status TEXT DEFAULT 'sql',
                promoted_at TEXT,
                source TEXT DEFAULT 'interaction'
            )",
            [],
        )
        .map_err(|e| e.to_string())?;

        // LoRA adapters
        conn.execute(
            "CREATE TABLE IF NOT EXISTS loras (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                rank INTEGER DEFAULT 4,
                alpha REAL DEFAULT 1.0,
                trained_on_items INTEGER DEFAULT 0,
                enabled INTEGER DEFAULT 0,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                epochs_trained INTEGER DEFAULT 0,
                loss REAL DEFAULT 0.0
            )",
            [],
        )
        .map_err(|e| e.to_string())?;

        // Training sessions
        conn.execute(
            "CREATE TABLE IF NOT EXISTS training_sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                started_at TEXT DEFAULT CURRENT_TIMESTAMP,
                completed_at TEXT,
                items_trained INTEGER DEFAULT 0,
                loss REAL DEFAULT 0.0,
                status TEXT DEFAULT 'pending',
                session_type TEXT DEFAULT 'promotion',
                notes TEXT
            )",
            [],
        )
        .map_err(|e| e.to_string())?;

        // User interactions
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

        // Baked snapshots
        conn.execute(
            "CREATE TABLE IF NOT EXISTS baked (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                version INTEGER DEFAULT 1,
                items_count INTEGER DEFAULT 0,
                baked_at TEXT DEFAULT CURRENT_TIMESTAMP,
                notes TEXT
            )",
            [],
        )
        .map_err(|e| e.to_string())?;

        // Indexes
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_knowledge_status ON knowledge(status)",
            [],
        )
        .ok();
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_knowledge_importance ON knowledge(importance)",
            [],
        )
        .ok();
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_knowledge_category ON knowledge(category)",
            [],
        )
        .ok();

        Ok(())
    }

    // ===== TRAINING PIPELINE =====

    // Step 1: Add to SQL (everything starts here)
    pub fn add_to_sql(
        &self,
        content: &str,
        category: &str,
        importance: i32,
        source: &str,
    ) -> Result<i64, String> {
        let _lock = self.lock.lock().map_err(|e| e.to_string())?;

        let conn = Connection::open(&self.sql_db_path).map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT INTO knowledge (content, category, importance, source) VALUES (?1, ?2, ?3, ?4)",
            params![content, category, importance, source],
        )
        .map_err(|e| e.to_string())?;

        Ok(conn.last_insert_rowid())
    }

    // Step 2: Record interaction for learning
    pub fn record_interaction(&self, user_msg: &str, ai_response: &str) -> Result<i64, String> {
        let _lock = self.lock.lock().map_err(|e| e.to_string())?;

        let conn = Connection::open(&self.sql_db_path).map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT INTO interactions (user_message, ai_response) VALUES (?1, ?2)",
            params![user_msg, ai_response],
        )
        .map_err(|e| e.to_string())?;

        Ok(conn.last_insert_rowid())
    }

    // Step 3: Give feedback - Director uses this to mark importance
    pub fn add_feedback(
        &self,
        item_id: i64,
        feedback: i32,
        new_importance: Option<i32>,
    ) -> Result<(), String> {
        let conn = Connection::open(&self.sql_db_path).map_err(|e| e.to_string())?;

        // Update feedback score
        conn.execute(
            "UPDATE knowledge SET feedback_score = ?1 WHERE id = ?2",
            params![feedback, item_id],
        )
        .map_err(|e| e.to_string())?;

        // If Director marked importance, update it
        if let Some(imp) = new_importance {
            conn.execute(
                "UPDATE knowledge SET importance = ?1 WHERE id = ?2",
                params![imp, item_id],
            )
            .map_err(|e| e.to_string())?;
        }

        // Update confidence based on feedback
        let delta = if feedback > 0 { 0.1 } else { -0.05 };
        conn.execute(
            "UPDATE knowledge SET confidence = MIN(1.0, MAX(0.0, confidence + ?1)),
             usage_count = usage_count + 1 WHERE id = ?2",
            params![delta, item_id],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    // Promote from SQL to RAG (important stuff)
    pub fn promote_to_rag(&self, min_importance: i32, max_items: i64) -> Result<i64, String> {
        let _lock = self.lock.lock().map_err(|e| e.to_string())?;

        let conn = Connection::open(&self.sql_db_path).map_err(|e| e.to_string())?;

        // Find important items in SQL to promote
        let updated = conn
            .execute(
                "UPDATE knowledge SET status = 'rag', promoted_at = CURRENT_TIMESTAMP 
             WHERE status = 'sql' AND importance >= ?1 
             ORDER BY importance DESC, confidence DESC LIMIT ?2",
                params![min_importance, max_items],
            )
            .map_err(|e| e.to_string())?;

        Ok(updated as i64)
    }

    // Get items for RAG context
    pub fn get_rag_context(&self, max_items: usize) -> Result<String, String> {
        let conn = Connection::open(&self.sql_db_path).map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare(
                "SELECT category, content FROM knowledge WHERE status = 'rag' 
             ORDER BY importance DESC, confidence DESC LIMIT ?1",
            )
            .map_err(|e| e.to_string())?;

        let items: Vec<(String, String)> = stmt
            .query_map(params![max_items as i64], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();

        if items.is_empty() {
            return Ok(String::new());
        }

        let mut context = format!("## {} Important Knowledge\n\n", self.ai_type);
        let mut current_category = String::new();

        for (category, content) in items {
            if category != current_category {
                context.push_str(&format!("### {}\n", category));
                current_category = category;
            }
            context.push_str(&format!("- {}\n", content));
        }

        Ok(context)
    }

    // Get SQL context (trivial stuff for completeness)
    pub fn get_sql_context(&self, max_items: usize) -> Result<String, String> {
        let conn = Connection::open(&self.sql_db_path).map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare(
                "SELECT content FROM knowledge WHERE status = 'sql' 
             ORDER BY usage_count DESC LIMIT ?1",
            )
            .map_err(|e| e.to_string())?;

        let items: Vec<String> = stmt
            .query_map(params![max_items as i64], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();

        if items.is_empty() {
            return Ok(String::new());
        }

        Ok(format!("## Related: {}\n", items.join(", ")))
    }

    // Create LoRA from RAG data
    pub fn create_lora(&self, name: &str, rank: i32, alpha: f32) -> Result<i64, String> {
        let conn = Connection::open(&self.sql_db_path).map_err(|e| e.to_string())?;

        // Count items that would be used
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM knowledge WHERE status IN ('rag', 'lora')",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        conn.execute(
            "INSERT INTO loras (name, rank, alpha, trained_on_items) VALUES (?1, ?2, ?3, ?4)",
            params![name, rank, alpha, count],
        )
        .map_err(|e| e.to_string())?;

        // Mark items as using this LoRA
        let lora_id = conn.last_insert_rowid();
        conn.execute(
            "UPDATE knowledge SET status = 'lora' WHERE status = 'rag'",
            [],
        )
        .map_err(|e| e.to_string())?;

        Ok(lora_id)
    }

    // Bake knowledge into model
    pub fn bake(&self, version: i32, notes: &str) -> Result<i64, String> {
        let _lock = self.lock.lock().map_err(|e| e.to_string())?;

        let conn = Connection::open(&self.sql_db_path).map_err(|e| e.to_string())?;

        // Count items being baked
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM knowledge WHERE status IN ('rag', 'lora')",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        // Mark as baked
        conn.execute(
            "UPDATE knowledge SET status = 'baked' WHERE status IN ('rag', 'lora')",
            [],
        )
        .map_err(|e| e.to_string())?;

        // Record bake
        conn.execute(
            "INSERT INTO baked (version, items_count, notes) VALUES (?1, ?2, ?3)",
            params![version, count, notes],
        )
        .map_err(|e| e.to_string())?;

        Ok(conn.last_insert_rowid())
    }

    // Get training stats
    pub fn get_stats(&self) -> Result<TrainingStats, String> {
        let conn = Connection::open(&self.sql_db_path).map_err(|e| e.to_string())?;

        let sql_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM knowledge WHERE status = 'sql'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let rag_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM knowledge WHERE status = 'rag'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let lora_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM knowledge WHERE status = 'lora'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let baked_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM knowledge WHERE status = 'baked'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);

        // Importance breakdown
        let mut breakdown = std::collections::HashMap::new();
        let mut stmt = conn
            .prepare("SELECT importance, COUNT(*) FROM knowledge GROUP BY importance")
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], |row| Ok((row.get::<_, i32>(0)?, row.get::<_, i64>(1)?)))
            .map_err(|e| e.to_string())?;

        for r in rows.flatten() {
            let label = match r.0 {
                0 => "trivial",
                1 => "important",
                2 => "critical",
                _ => "unknown",
            };
            breakdown.insert(label.to_string(), r.1);
        }

        // Thresholds: 1000 sql → promote to rag, 100 rag → create lora, 5 lora → bake
        Ok(TrainingStats {
            ai_type: self.ai_type.clone(),
            sql_items: sql_count,
            rag_items: rag_count,
            lora_items: lora_count,
            baked_items: baked_count,
            total_items: sql_count + rag_count + lora_count + baked_count,
            importance_breakdown: breakdown,
            should_promote_to_rag: sql_count >= 1000,
            should_create_lora: rag_count >= 100,
            should_bake: lora_count >= 5,
        })
    }

    // Get all knowledge items
    pub fn get_knowledge(&self, status: Option<&str>) -> Result<Vec<KnowledgeItem>, String> {
        let conn = Connection::open(&self.sql_db_path).map_err(|e| e.to_string())?;

        let query = match status {
            Some(s) => format!(
                "SELECT id, content, category, importance, confidence, usage_count, created_at, status 
                 FROM knowledge WHERE status = '{}' ORDER BY importance DESC, confidence DESC", s
            ),
            None => String::from(
                "SELECT id, content, category, importance, confidence, usage_count, created_at, status 
                 FROM knowledge ORDER BY importance DESC, confidence DESC"
            ),
        };

        let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;

        let items = stmt
            .query_map([], |row| {
                Ok(KnowledgeItem {
                    id: row.get(0)?,
                    content: row.get(1)?,
                    category: row.get(2)?,
                    importance: row.get(3)?,
                    confidence: row.get(4)?,
                    usage_count: row.get(5)?,
                    created_at: row.get(6)?,
                    status: row.get(7)?,
                })
            })
            .map_err(|e| e.to_string())?;

        items
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn get_loras(&self) -> Result<Vec<LoraConfig>, String> {
        let conn = Connection::open(&self.sql_db_path).map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare(
                "SELECT id, name, rank, alpha, trained_on_items, enabled, created_at FROM loras",
            )
            .map_err(|e| e.to_string())?;

        let loras = stmt
            .query_map([], |row| {
                Ok(LoraConfig {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    rank: row.get(2)?,
                    alpha: row.get(3)?,
                    trained_on: row.get(4)?,
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
        let conn = Connection::open(&self.sql_db_path).map_err(|e| e.to_string())?;

        conn.execute(
            "UPDATE loras SET enabled = ?1 WHERE id = ?2",
            params![enabled as i32, id],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
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
                sql_items: 0,
                rag_items: 0,
                lora_items: 0,
                baked_items: 0,
                total_items: 0,
                importance_breakdown: std::collections::HashMap::new(),
                should_promote_to_rag: false,
                should_create_lora: false,
                should_bake: false,
            }),
            self.programmer.get_stats().unwrap_or(TrainingStats {
                ai_type: "programmer".to_string(),
                sql_items: 0,
                rag_items: 0,
                lora_items: 0,
                baked_items: 0,
                total_items: 0,
                importance_breakdown: std::collections::HashMap::new(),
                should_promote_to_rag: false,
                should_create_lora: false,
                should_bake: false,
            }),
            self.vision.get_stats().unwrap_or(TrainingStats {
                ai_type: "vision".to_string(),
                sql_items: 0,
                rag_items: 0,
                lora_items: 0,
                baked_items: 0,
                total_items: 0,
                importance_breakdown: std::collections::HashMap::new(),
                should_promote_to_rag: false,
                should_create_lora: false,
                should_bake: false,
            }),
        ]
    }
}

impl Default for TrainingManager {
    fn default() -> Self {
        Self::new()
    }
}
