use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Message {
    pub id: Option<Uuid>,
    pub name: String,
    pub email: String,
    pub subject: String,
    pub message: String,
    pub read: bool,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateMessage {
    pub name: String,
    pub email: String,
    pub subject: String,
    pub message: String,
}

impl Message {
    pub async fn all() -> AppResult<Vec<Self>> {
        let pool = get_pool();
        let rows = sqlx::query_as::<_, Self>(
            "SELECT id, name, email, subject, message, read, created_at FROM messages ORDER BY created_at DESC",
        )
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    pub async fn by_id(id: &str) -> AppResult<Option<Self>> {
        let id = Uuid::parse_str(id)?;
        let pool = get_pool();
        let row = sqlx::query_as::<_, Self>(
            "SELECT id, name, email, subject, message, read, created_at FROM messages WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;
        Ok(row)
    }

    pub async fn create(data: CreateMessage) -> AppResult<Self> {
        let pool = get_pool();

        let row = sqlx::query_as::<_, Self>(
            r#"
            INSERT INTO messages (name, email, subject, message)
            VALUES ($1, $2, $3, $4)
            RETURNING id, name, email, subject, message, read, created_at
            "#,
        )
        .bind(&data.name)
        .bind(&data.email)
        .bind(&data.subject)
        .bind(&data.message)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    pub async fn mark_read(id: &str) -> AppResult<Self> {
        let id = Uuid::parse_str(id)?;
        let pool = get_pool();
        let row = sqlx::query_as::<_, Self>(
            "UPDATE messages SET read = true WHERE id = $1 RETURNING id, name, email, subject, message, read, created_at",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        row.ok_or_else(|| AppError::NotFound("Message not found".to_string()))
    }

    pub async fn delete(id: &str) -> AppResult<()> {
        let id = Uuid::parse_str(id)?;
        let pool = get_pool();
        let result = sqlx::query("DELETE FROM messages WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Message not found".to_string()));
        }
        Ok(())
    }

    pub async fn count_unread() -> AppResult<i64> {
        let pool = get_pool();
        let row: (i64,) = sqlx::query_as("SELECT count(*) FROM messages WHERE read = false")
            .fetch_one(pool)
            .await?;
        Ok(row.0)
    }

    pub async fn recent(limit: usize) -> AppResult<Vec<Self>> {
        let pool = get_pool();
        let limit = limit as i64;
        let rows = sqlx::query_as::<_, Self>(
            "SELECT id, name, email, subject, message, read, created_at FROM messages ORDER BY created_at DESC LIMIT $1",
        )
        .bind(limit)
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }
}
