use anyhow::Result;
use sqlx::{Row, sqlite::SqlitePool};
use tracing::info;

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new() -> Result<Self> {
        let database_url =
            std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:main.db".to_string());

        let pool = SqlitePool::connect(&database_url).await?;

        Ok(Database { pool })
    }

    pub async fn init(&self) -> Result<()> {
        // Create the shard_stats table if it doesn't exist
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS shard_stats (
                shard_id INT PRIMARY KEY,
                server_count INT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                mem REAL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create the process_stats table for multi-process tracking
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS process_stats (
                process_id TEXT PRIMARY KEY,
                shard_start INT,
                shard_count INT,
                total_shards INT,
                server_count INT,
                memory_mb REAL,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        info!("Database initialized successfully");
        Ok(())
    }

    pub async fn update_shard_stats(
        &self,
        shard_id: i32,
        server_count: i32,
        memory_mb: f64,
    ) -> Result<()> {
        // Insert or replace the shard stats
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO shard_stats (shard_id, server_count, timestamp, mem)
            VALUES (?, ?, CURRENT_TIMESTAMP, ?)
            "#,
        )
        .bind(shard_id)
        .bind(server_count)
        .bind(memory_mb)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // New function to update process stats for multi-process sharding
    pub async fn update_process_stats(
        &self,
        process_id: &str,
        shard_start: i32,
        shard_count: i32,
        total_shards: i32,
        server_count: i32,
        memory_mb: f64,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO process_stats 
            (process_id, shard_start, shard_count, total_shards, server_count, memory_mb, timestamp)
            VALUES (?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
            "#,
        )
        .bind(process_id)
        .bind(shard_start)
        .bind(shard_count)
        .bind(total_shards)
        .bind(server_count)
        .bind(memory_mb)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_shard_stats(&self, shard_id: i32) -> Result<Option<ShardStats>> {
        let row = sqlx::query(
            "SELECT shard_id, server_count, timestamp, mem FROM shard_stats WHERE shard_id = ?",
        )
        .bind(shard_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(ShardStats {
                shard_id: row.get("shard_id"),
                server_count: row.get("server_count"),
                timestamp: row.get("timestamp"),
                memory_mb: row.get("mem"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_all_shard_stats(&self) -> Result<Vec<ShardStats>> {
        let rows = sqlx::query(
            "SELECT shard_id, server_count, timestamp, mem FROM shard_stats ORDER BY timestamp DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        let mut stats = Vec::new();
        for row in rows {
            stats.push(ShardStats {
                shard_id: row.get("shard_id"),
                server_count: row.get("server_count"),
                timestamp: row.get("timestamp"),
                memory_mb: row.get("mem"),
            });
        }

        Ok(stats)
    }

    // New function to get all process stats
    pub async fn get_all_process_stats(&self) -> Result<Vec<ProcessStats>> {
        let rows = sqlx::query(
            r#"
            SELECT process_id, shard_start, shard_count, total_shards, server_count, memory_mb, timestamp 
            FROM process_stats 
            ORDER BY timestamp DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut stats = Vec::new();
        for row in rows {
            stats.push(ProcessStats {
                process_id: row.get("process_id"),
                shard_start: row.get("shard_start"),
                shard_count: row.get("shard_count"),
                total_shards: row.get("total_shards"),
                server_count: row.get("server_count"),
                memory_mb: row.get("memory_mb"),
                timestamp: row.get("timestamp"),
            });
        }

        Ok(stats)
    }

    // Clean up old process stats (remove entries older than 30 minutes)
    pub async fn cleanup_old_process_stats(&self) -> Result<()> {
        sqlx::query("DELETE FROM process_stats WHERE timestamp < datetime('now', '-30 minutes')")
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ShardStats {
    pub shard_id: i32,
    pub server_count: i32,
    pub timestamp: String,
    pub memory_mb: f64,
}

#[derive(Debug, Clone)]
pub struct ProcessStats {
    pub process_id: String,
    pub shard_start: i32,
    pub shard_count: i32,
    pub total_shards: i32,
    pub server_count: i32,
    pub memory_mb: f64,
    pub timestamp: String,
}
