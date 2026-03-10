use chrono::Utc;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub struct Database {
    conn: Connection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatHistoryEntry {
    pub id: i64,
    pub role: String,
    pub content: String,
    pub timestamp: String,
    pub ai_mode: String,
    pub request_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagDocument {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub source: String,
    pub created_at: String,
    pub ai_type: String,
}

impl Database {
    pub fn new() -> Result<Self> {
        let db_path = Self::get_db_path();

        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let conn = Connection::open(&db_path)?;

        let db = Self { conn };
        db.init_tables()?;

        Ok(db)
    }

    fn get_db_path() -> PathBuf {
        if let Some(proj_dirs) = directories::ProjectDirs::from("com", "kaelos", "Kael") {
            proj_dirs.data_dir().join("kael.db")
        } else {
            PathBuf::from("kael.db")
        }
    }

    fn init_tables(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS chat_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                ai_mode TEXT NOT NULL,
                request_type TEXT
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS rag_documents (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                source TEXT NOT NULL,
                created_at TEXT NOT NULL,
                ai_type TEXT NOT NULL
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS lora_configs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                ai_type TEXT NOT NULL,
                description TEXT,
                model_path TEXT,
                enabled INTEGER DEFAULT 0,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        Ok(())
    }

    pub fn add_message(
        &self,
        role: &str,
        content: &str,
        ai_mode: &str,
        request_type: Option<&str>,
    ) -> Result<i64> {
        let timestamp = Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT INTO chat_history (role, content, timestamp, ai_mode, request_type) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![role, content, timestamp, ai_mode, request_type],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_chat_history(&self, ai_mode: &str, limit: i64) -> Result<Vec<ChatHistoryEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, role, content, timestamp, ai_mode, request_type 
             FROM chat_history 
             WHERE ai_mode = ?1 
             ORDER BY timestamp DESC 
             LIMIT ?2",
        )?;

        let entries = stmt.query_map(params![ai_mode, limit], |row| {
            Ok(ChatHistoryEntry {
                id: row.get(0)?,
                role: row.get(1)?,
                content: row.get(2)?,
                timestamp: row.get(3)?,
                ai_mode: row.get(4)?,
                request_type: row.get(5)?,
            })
        })?;

        entries.collect()
    }

    pub fn clear_chat_history(&self, ai_mode: Option<&str>) -> Result<()> {
        match ai_mode {
            Some(mode) => {
                self.conn
                    .execute("DELETE FROM chat_history WHERE ai_mode = ?1", params![mode])?;
            }
            None => {
                self.conn.execute("DELETE FROM chat_history", [])?;
            }
        }
        Ok(())
    }

    pub fn add_document(
        &self,
        title: &str,
        content: &str,
        source: &str,
        ai_type: &str,
    ) -> Result<i64> {
        let timestamp = Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT INTO rag_documents (title, content, source, created_at, ai_type) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![title, content, source, timestamp, ai_type],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    pub fn search_documents(
        &self,
        query: &str,
        ai_type: &str,
        limit: i64,
    ) -> Result<Vec<RagDocument>> {
        let search_pattern = format!("%{}%", query);

        let mut stmt = self.conn.prepare(
            "SELECT id, title, content, source, created_at, ai_type 
             FROM rag_documents 
             WHERE ai_type = ?1 AND (title LIKE ?2 OR content LIKE ?2)
             ORDER BY created_at DESC 
             LIMIT ?3",
        )?;

        let docs = stmt.query_map(params![ai_type, search_pattern, limit], |row| {
            Ok(RagDocument {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                source: row.get(3)?,
                created_at: row.get(4)?,
                ai_type: row.get(5)?,
            })
        })?;

        docs.collect()
    }

    pub fn get_all_documents(&self, ai_type: &str) -> Result<Vec<RagDocument>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, content, source, created_at, ai_type 
             FROM rag_documents 
             WHERE ai_type = ?1
             ORDER BY created_at DESC",
        )?;

        let docs = stmt.query_map(params![ai_type], |row| {
            Ok(RagDocument {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                source: row.get(3)?,
                created_at: row.get(4)?,
                ai_type: row.get(5)?,
            })
        })?;

        docs.collect()
    }

    pub fn delete_document(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM rag_documents WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn add_lora(
        &self,
        name: &str,
        ai_type: &str,
        description: Option<&str>,
        model_path: Option<&str>,
    ) -> Result<i64> {
        let timestamp = Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT INTO lora_configs (name, ai_type, description, model_path, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![name, ai_type, description, model_path, timestamp],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_loras(&self, ai_type: Option<&str>) -> Result<Vec<LoraConfig>> {
        let mut loras = Vec::new();

        if let Some(t) = ai_type {
            let mut stmt = self.conn.prepare(
                "SELECT id, name, ai_type, description, model_path, enabled, created_at FROM lora_configs WHERE ai_type = ?1"
            )?;
            let rows = stmt.query_map(params![t], |row| {
                Ok(LoraConfig {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    ai_type: row.get(2)?,
                    description: row.get(3)?,
                    model_path: row.get(4)?,
                    enabled: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })?;
            for row in rows {
                loras.push(row?);
            }
        } else {
            let mut stmt = self.conn.prepare(
                "SELECT id, name, ai_type, description, model_path, enabled, created_at FROM lora_configs"
            )?;
            let rows = stmt.query_map([], |row| {
                Ok(LoraConfig {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    ai_type: row.get(2)?,
                    description: row.get(3)?,
                    model_path: row.get(4)?,
                    enabled: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })?;
            for row in rows {
                loras.push(row?);
            }
        }

        Ok(loras)
    }

    pub fn enable_lora(&self, id: i64, enabled: bool) -> Result<()> {
        self.conn.execute(
            "UPDATE lora_configs SET enabled = ?1 WHERE id = ?2",
            params![enabled as i32, id],
        )?;
        Ok(())
    }

    pub fn delete_lora(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM lora_configs WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn get_stats(&self) -> Result<DatabaseStats> {
        let chat_count: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM chat_history", [], |row| row.get(0))?;

        let doc_count: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM rag_documents", [], |row| row.get(0))?;

        let lora_count: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM lora_configs", [], |row| row.get(0))?;

        Ok(DatabaseStats {
            chat_messages: chat_count,
            rag_documents: doc_count,
            lora_configs: lora_count,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoraConfig {
    pub id: i64,
    pub name: String,
    pub ai_type: String,
    pub description: Option<String>,
    pub model_path: Option<String>,
    pub enabled: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub chat_messages: i64,
    pub rag_documents: i64,
    pub lora_configs: i64,
}
