use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Row};
use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clip {
    pub id: String,
    pub content: String,
    pub clip_type: String,
    pub created_at: DateTime<Utc>,
    pub file_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statistics {
    pub total_clips: usize,
    pub text_clips: usize,
    pub file_clips: usize,
    pub oldest_clip: String,
    pub newest_clip: String,
    pub db_size_kb: usize,
}

impl From<&Row<'_>> for Clip {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id").unwrap_or_default(),
            content: row.get("content").unwrap_or_default(),
            clip_type: row.get("clip_type").unwrap_or_default(),
            created_at: DateTime::from_timestamp(
                row.get::<_, i64>("created_at").unwrap_or(0),
                0,
            ).unwrap_or_else(|| Utc::now()),
            file_path: row.get("file_path").ok(),
        }
    }
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub async fn new() -> Result<Self> {
        let db_path = Self::get_db_path()?;
        
        // Create directory if it doesn't exist
        if let Some(parent) = Path::new(&db_path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let conn = Connection::open(&db_path)?;
        let db = Database { conn };
        db.init_tables().await?;
        Ok(db)
    }

    fn get_db_path() -> Result<String> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        let db_path = home.join(".clipq").join("clipboard.db");
        Ok(db_path.to_string_lossy().to_string())
    }

    async fn init_tables(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS clips (
                id TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                clip_type TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                file_path TEXT
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS clip_tags (
                clip_id TEXT NOT NULL,
                tag_id INTEGER NOT NULL,
                PRIMARY KEY (clip_id, tag_id),
                FOREIGN KEY (clip_id) REFERENCES clips(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_created_at ON clips(created_at DESC)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_content ON clips(content)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_clip_type ON clips(clip_type)",
            [],
        )?;

        Ok(())
    }

    pub async fn add_clip(&mut self, content: &str, clip_type: &str) -> Result<()> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().timestamp();
        
        self.conn.execute(
            "INSERT INTO clips (id, content, clip_type, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![id, content, clip_type, now],
        )?;

        Ok(())
    }

    pub async fn add_file_clip(&mut self, file_path: &str) -> Result<()> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().timestamp();
        
        self.conn.execute(
            "INSERT INTO clips (id, content, clip_type, created_at, file_path) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, file_path, "file", now, file_path],
        )?;

        Ok(())
    }

    pub async fn get_recent_clips(&self, limit: usize) -> Result<Vec<Clip>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content, clip_type, created_at, file_path FROM clips 
             ORDER BY created_at DESC LIMIT ?1"
        )?;
        
        let clip_iter = stmt.query_map(params![limit], |row| {
            Ok(Clip::from(row))
        })?;

        let mut clips = Vec::new();
        for clip in clip_iter {
            clips.push(clip?);
        }

        Ok(clips)
    }

    pub async fn get_clip_by_id(&self, id: &str) -> Result<Option<Clip>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content, clip_type, created_at, file_path FROM clips WHERE id = ?1"
        )?;
        
        let mut rows = stmt.query_map(params![id], |row| {
            Ok(Clip::from(row))
        })?;

        Ok(rows.next().transpose()?)
    }

    pub async fn clear_history(&mut self) -> Result<()> {
        self.conn.execute("DELETE FROM clips", [])?;
        Ok(())
    }

    pub async fn trim_history(&mut self, max_clips: usize) -> Result<()> {
        let mut stmt = self.conn.prepare(
            "DELETE FROM clips WHERE id NOT IN (
                SELECT id FROM clips ORDER BY created_at DESC LIMIT ?1
            )"
        )?;
        
        stmt.execute(params![max_clips])?;
        Ok(())
    }

    pub async fn search_clips(&self, query: &str, limit: usize) -> Result<Vec<Clip>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content, clip_type, created_at, file_path FROM clips 
             WHERE content LIKE ?1 
             ORDER BY created_at DESC LIMIT ?2"
        )?;
        
        let search_pattern = format!("%{}%", query);
        let clip_iter = stmt.query_map(params![search_pattern, limit], |row| {
            Ok(Clip::from(row))
        })?;

        let mut clips = Vec::new();
        for clip in clip_iter {
            clips.push(clip?);
        }

        Ok(clips)
    }

    pub async fn get_all_clips(&self) -> Result<Vec<Clip>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content, clip_type, created_at, file_path FROM clips 
             ORDER BY created_at DESC"
        )?;
        
        let clip_iter = stmt.query_map([], |row| {
            Ok(Clip::from(row))
        })?;

        let mut clips = Vec::new();
        for clip in clip_iter {
            clips.push(clip?);
        }

        Ok(clips)
    }

    pub async fn get_statistics(&self) -> Result<Statistics> {
        let mut stmt = self.conn.prepare("SELECT COUNT(*) FROM clips")?;
        let total_clips: usize = stmt.query_row([], |row| row.get(0))?;

        let mut stmt = self.conn.prepare("SELECT COUNT(*) FROM clips WHERE clip_type = 'text'")?;
        let text_clips: usize = stmt.query_row([], |row| row.get(0))?;

        let mut stmt = self.conn.prepare("SELECT COUNT(*) FROM clips WHERE clip_type = 'file'")?;
        let file_clips: usize = stmt.query_row([], |row| row.get(0))?;

        let mut stmt = self.conn.prepare("SELECT MIN(created_at) FROM clips")?;
        let oldest_timestamp: i64 = stmt.query_row([], |row| row.get(0)).unwrap_or(0);
        let oldest_clip = DateTime::from_timestamp(oldest_timestamp, 0)
            .unwrap_or_else(|| Utc::now())
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();

        let mut stmt = self.conn.prepare("SELECT MAX(created_at) FROM clips")?;
        let newest_timestamp: i64 = stmt.query_row([], |row| row.get(0)).unwrap_or(0);
        let newest_clip = DateTime::from_timestamp(newest_timestamp, 0)
            .unwrap_or_else(|| Utc::now())
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();

        // Get database file size
        let db_path = Self::get_db_path()?;
        let db_size = std::fs::metadata(&db_path)
            .map(|m| m.len() as usize / 1024)
            .unwrap_or(0);

        Ok(Statistics {
            total_clips,
            text_clips,
            file_clips,
            oldest_clip,
            newest_clip,
            db_size_kb: db_size,
        })
    }

    pub async fn add_tag_to_clip(&mut self, clip_id: &str, tag_name: &str) -> Result<()> {
        // First, ensure the tag exists
        let mut stmt = self.conn.prepare("INSERT OR IGNORE INTO tags (name) VALUES (?1)")?;
        stmt.execute(params![tag_name])?;

        // Get the tag ID
        let mut stmt = self.conn.prepare("SELECT id FROM tags WHERE name = ?1")?;
        let tag_id: i64 = stmt.query_row(params![tag_name], |row| row.get(0))?;

        // Add the relationship
        let mut stmt = self.conn.prepare("INSERT OR IGNORE INTO clip_tags (clip_id, tag_id) VALUES (?1, ?2)")?;
        stmt.execute(params![clip_id, tag_id])?;

        Ok(())
    }

    pub async fn remove_tag_from_clip(&mut self, clip_id: &str, tag_name: &str) -> Result<()> {
        let mut stmt = self.conn.prepare(
            "DELETE FROM clip_tags WHERE clip_id = ?1 AND tag_id = (
                SELECT id FROM tags WHERE name = ?2
            )"
        )?;
        stmt.execute(params![clip_id, tag_name])?;

        Ok(())
    }

    pub async fn get_clip_tags(&self, clip_id: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT t.name FROM tags t 
             JOIN clip_tags ct ON t.id = ct.tag_id 
             WHERE ct.clip_id = ?1"
        )?;
        
        let tag_iter = stmt.query_map(params![clip_id], |row| {
            Ok(row.get::<_, String>(0)?)
        })?;

        let mut tags = Vec::new();
        for tag in tag_iter {
            tags.push(tag?);
        }

        Ok(tags)
    }

    pub async fn get_clips_by_tag(&self, tag_name: &str) -> Result<Vec<Clip>> {
        let mut stmt = self.conn.prepare(
            "SELECT c.id, c.content, c.clip_type, c.created_at, c.file_path 
             FROM clips c 
             JOIN clip_tags ct ON c.id = ct.clip_id 
             JOIN tags t ON ct.tag_id = t.id 
             WHERE t.name = ?1 
             ORDER BY c.created_at DESC"
        )?;
        
        let clip_iter = stmt.query_map(params![tag_name], |row| {
            Ok(Clip::from(row))
        })?;

        let mut clips = Vec::new();
        for clip in clip_iter {
            clips.push(clip?);
        }

        Ok(clips)
    }

    pub async fn backup(&self, output_path: &str) -> Result<()> {
        let db_path = Self::get_db_path()?;
        std::fs::copy(&db_path, output_path)?;
        Ok(())
    }

    pub async fn restore(&mut self, input_path: &str) -> Result<()> {
        let db_path = Self::get_db_path()?;
        std::fs::copy(input_path, &db_path)?;
        Ok(())
    }
}