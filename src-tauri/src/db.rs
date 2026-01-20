// Database module for ShotAuto
use rusqlite::{Connection, Result, params};
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Application configuration stored in database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub youtube_api_key: Option<String>,
    pub telegram_bot_token: Option<String>,
    pub telegram_chat_id: Option<String>,
    pub ollama_endpoint: String,
    pub poll_interval_secs: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            youtube_api_key: None,
            telegram_bot_token: None,
            telegram_chat_id: None,
            ollama_endpoint: "http://localhost:11434".to_string(),
            poll_interval_secs: 300, // 5 minutes
        }
    }
}

/// Trend data from YouTube
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trend {
    pub id: Option<i64>,
    pub video_id: String,
    pub title: String,
    pub channel: Option<String>,
    pub views: Option<i64>,
    pub category: Option<String>,
    pub fetched_at: DateTime<Utc>,
}

/// Job status enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Generating,
    Rendering,
    Done,
    Failed,
}

impl JobStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            JobStatus::Pending => "pending",
            JobStatus::Generating => "generating",
            JobStatus::Rendering => "rendering",
            JobStatus::Done => "done",
            JobStatus::Failed => "failed",
        }
    }
    
    pub fn from_str(s: &str) -> Self {
        match s {
            "pending" => JobStatus::Pending,
            "generating" => JobStatus::Generating,
            "rendering" => JobStatus::Rendering,
            "done" => JobStatus::Done,
            "failed" => JobStatus::Failed,
            _ => JobStatus::Pending,
        }
    }
}

/// Job in the processing queue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: Option<i64>,
    pub trend_id: i64,
    pub status: JobStatus,
    pub priority: i32,
    pub retry_count: i32,
    pub error_msg: Option<String>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
}

/// Generated short video
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Short {
    pub id: Option<i64>,
    pub job_id: i64,
    pub script: Option<String>,
    pub audio_path: Option<String>,
    pub video_path: Option<String>,
    pub duration_sec: Option<f64>,
    pub telegram_sent: bool,
}

/// Database connection wrapper
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Create a new database connection
    pub fn new(path: PathBuf) -> Result<Self> {
        let conn = Connection::open(&path)?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }
    
    /// Initialize database schema
    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            -- Configuration table
            CREATE TABLE IF NOT EXISTS config (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            
            -- Trends from YouTube
            CREATE TABLE IF NOT EXISTS trends (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                video_id TEXT UNIQUE NOT NULL,
                title TEXT NOT NULL,
                channel TEXT,
                views INTEGER,
                category TEXT,
                fetched_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );
            
            -- Processing jobs queue
            CREATE TABLE IF NOT EXISTS jobs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                trend_id INTEGER REFERENCES trends(id),
                status TEXT DEFAULT 'pending' 
                    CHECK(status IN ('pending','generating','rendering','done','failed')),
                priority INTEGER DEFAULT 0,
                retry_count INTEGER DEFAULT 0,
                error_msg TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                started_at TIMESTAMP,
                finished_at TIMESTAMP
            );
            
            -- Generated shorts
            CREATE TABLE IF NOT EXISTS shorts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                job_id INTEGER REFERENCES jobs(id),
                script TEXT,
                audio_path TEXT,
                video_path TEXT,
                duration_sec REAL,
                telegram_sent BOOLEAN DEFAULT 0
            );
            
            -- Performance metrics
            CREATE TABLE IF NOT EXISTS metrics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                job_id INTEGER REFERENCES jobs(id),
                stage TEXT,
                duration_ms INTEGER,
                recorded_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );
            
            -- Create indexes
            CREATE INDEX IF NOT EXISTS idx_jobs_status ON jobs(status);
            CREATE INDEX IF NOT EXISTS idx_trends_video_id ON trends(video_id);
            "#
        )?;
        Ok(())
    }
    
    // ==================== Config CRUD ====================
    
    /// Get a config value
    pub fn get_config(&self, key: &str) -> Result<Option<String>> {
        let mut stmt = self.conn.prepare("SELECT value FROM config WHERE key = ?")?;
        let mut rows = stmt.query(params![key])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }
    
    /// Set a config value
    pub fn set_config(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO config (key, value) VALUES (?, ?)",
            params![key, value],
        )?;
        Ok(())
    }
    
    /// Load full config
    pub fn load_config(&self) -> Result<Config> {
        Ok(Config {
            youtube_api_key: self.get_config("youtube_api_key")?,
            telegram_bot_token: self.get_config("telegram_bot_token")?,
            telegram_chat_id: self.get_config("telegram_chat_id")?,
            ollama_endpoint: self.get_config("ollama_endpoint")?
                .unwrap_or_else(|| "http://localhost:11434".to_string()),
            poll_interval_secs: self.get_config("poll_interval_secs")?
                .and_then(|s| s.parse().ok())
                .unwrap_or(300),
        })
    }
    
    /// Save full config
    pub fn save_config(&self, config: &Config) -> Result<()> {
        if let Some(ref key) = config.youtube_api_key {
            self.set_config("youtube_api_key", key)?;
        }
        if let Some(ref token) = config.telegram_bot_token {
            self.set_config("telegram_bot_token", token)?;
        }
        if let Some(ref chat_id) = config.telegram_chat_id {
            self.set_config("telegram_chat_id", chat_id)?;
        }
        self.set_config("ollama_endpoint", &config.ollama_endpoint)?;
        self.set_config("poll_interval_secs", &config.poll_interval_secs.to_string())?;
        Ok(())
    }
    
    // ==================== Trends CRUD ====================
    
    /// Insert a new trend (ignores duplicates)
    pub fn insert_trend(&self, trend: &Trend) -> Result<i64> {
        self.conn.execute(
            "INSERT OR IGNORE INTO trends (video_id, title, channel, views, category, fetched_at) 
             VALUES (?, ?, ?, ?, ?, ?)",
            params![
                trend.video_id,
                trend.title,
                trend.channel,
                trend.views,
                trend.category,
                trend.fetched_at.to_rfc3339(),
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }
    
    /// Get trend by video_id
    pub fn get_trend_by_video_id(&self, video_id: &str) -> Result<Option<Trend>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, video_id, title, channel, views, category, fetched_at FROM trends WHERE video_id = ?"
        )?;
        let mut rows = stmt.query(params![video_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Trend {
                id: Some(row.get(0)?),
                video_id: row.get(1)?,
                title: row.get(2)?,
                channel: row.get(3)?,
                views: row.get(4)?,
                category: row.get(5)?,
                fetched_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            }))
        } else {
            Ok(None)
        }
    }
    
    // ==================== Jobs CRUD ====================
    
    /// Create a new job for a trend
    pub fn create_job(&self, trend_id: i64, priority: i32) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO jobs (trend_id, status, priority) VALUES (?, 'pending', ?)",
            params![trend_id, priority],
        )?;
        Ok(self.conn.last_insert_rowid())
    }
    
    /// Get next pending job
    pub fn get_next_pending_job(&self) -> Result<Option<(Job, Trend)>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT j.id, j.trend_id, j.status, j.priority, j.retry_count, j.error_msg, 
                   j.created_at, j.started_at, j.finished_at,
                   t.id, t.video_id, t.title, t.channel, t.views, t.category, t.fetched_at
            FROM jobs j
            JOIN trends t ON j.trend_id = t.id
            WHERE j.status = 'pending'
            ORDER BY j.priority DESC, j.created_at ASC
            LIMIT 1
            "#
        )?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            let job = Job {
                id: Some(row.get(0)?),
                trend_id: row.get(1)?,
                status: JobStatus::from_str(&row.get::<_, String>(2)?),
                priority: row.get(3)?,
                retry_count: row.get(4)?,
                error_msg: row.get(5)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                started_at: row.get::<_, Option<String>>(7)?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                finished_at: row.get::<_, Option<String>>(8)?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
            };
            let trend = Trend {
                id: Some(row.get(9)?),
                video_id: row.get(10)?,
                title: row.get(11)?,
                channel: row.get(12)?,
                views: row.get(13)?,
                category: row.get(14)?,
                fetched_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(15)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            };
            Ok(Some((job, trend)))
        } else {
            Ok(None)
        }
    }
    
    /// Update job status
    pub fn update_job_status(&self, job_id: i64, status: JobStatus, error_msg: Option<&str>) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        match status {
            JobStatus::Generating | JobStatus::Rendering => {
                self.conn.execute(
                    "UPDATE jobs SET status = ?, started_at = ? WHERE id = ?",
                    params![status.as_str(), now, job_id],
                )?;
            }
            JobStatus::Done | JobStatus::Failed => {
                self.conn.execute(
                    "UPDATE jobs SET status = ?, finished_at = ?, error_msg = ? WHERE id = ?",
                    params![status.as_str(), now, error_msg, job_id],
                )?;
            }
            _ => {
                self.conn.execute(
                    "UPDATE jobs SET status = ? WHERE id = ?",
                    params![status.as_str(), job_id],
                )?;
            }
        }
        Ok(())
    }
    
    // ==================== Stats ====================
    
    /// Get dashboard statistics
    pub fn get_stats(&self) -> Result<DashboardStats> {
        let trends_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM trends", [], |row| row.get(0)
        )?;
        let pending_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM jobs WHERE status = 'pending'", [], |row| row.get(0)
        )?;
        let done_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM jobs WHERE status = 'done'", [], |row| row.get(0)
        )?;
        let failed_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM jobs WHERE status = 'failed'", [], |row| row.get(0)
        )?;
        
        Ok(DashboardStats {
            total_trends: trends_count,
            pending_jobs: pending_count,
            completed_jobs: done_count,
            failed_jobs: failed_count,
        })
    }
}

/// Dashboard statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    pub total_trends: i64,
    pub pending_jobs: i64,
    pub completed_jobs: i64,
    pub failed_jobs: i64,
}
