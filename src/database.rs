use anyhow::Result;
use sqlx::{sqlite::SqlitePool, Row};
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
            "SELECT shard_id, server_count, timestamp, mem FROM shard_stats ORDER BY shard_id",
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
}

#[derive(Debug, Clone)]
pub struct ShardStats {
    pub shard_id: i32,
    pub server_count: i32,
    pub timestamp: String,
    pub memory_mb: f64,
}
