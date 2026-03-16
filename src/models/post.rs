use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use pulldown_cmark::{html, Parser};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Post {
    pub id: Option<Uuid>,
    pub title: String,
    pub slug: String,
    pub excerpt: String,
    pub content: String,
    pub tags: Vec<String>,
    pub published: bool,
    pub reading_time: i32,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePost {
    pub title: String,
    pub excerpt: String,
    pub content: String,
    pub tags: Vec<String>,
    pub published: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePost {
    pub title: Option<String>,
    pub excerpt: Option<String>,
    pub content: Option<String>,
    pub tags: Option<Vec<String>>,
    pub published: Option<bool>,
}

impl Post {
    pub fn content_html(&self) -> String {
        let parser = Parser::new(&self.content);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);
        html_output
    }

    fn calculate_reading_time(content: &str) -> i32 {
        let word_count = content.split_whitespace().count();
        ((word_count as f64 / 200.0).ceil() as i32).max(1)
    }

    pub async fn all() -> AppResult<Vec<Self>> {
        let pool = get_pool();
        let rows = sqlx::query_as::<_, Self>(
            "SELECT id, title, slug, excerpt, content, tags, published, reading_time, created_at, updated_at FROM posts ORDER BY created_at DESC",
        )
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    pub async fn published() -> AppResult<Vec<Self>> {
        let pool = get_pool();
        let rows = sqlx::query_as::<_, Self>(
            "SELECT id, title, slug, excerpt, content, tags, published, reading_time, created_at, updated_at FROM posts WHERE published = true ORDER BY created_at DESC",
        )
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    pub async fn by_tag(tag: &str) -> AppResult<Vec<Self>> {
        let pool = get_pool();
        let rows = sqlx::query_as::<_, Self>(
            "SELECT id, title, slug, excerpt, content, tags, published, reading_time, created_at, updated_at FROM posts WHERE $1 = ANY(tags) AND published = true ORDER BY created_at DESC",
        )
        .bind(tag)
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    pub async fn by_slug(slug: &str) -> AppResult<Option<Self>> {
        let pool = get_pool();
        let row = sqlx::query_as::<_, Self>(
            "SELECT id, title, slug, excerpt, content, tags, published, reading_time, created_at, updated_at FROM posts WHERE slug = $1 LIMIT 1",
        )
        .bind(slug)
        .fetch_optional(pool)
        .await?;
        Ok(row)
    }

    pub async fn by_id(id: &str) -> AppResult<Option<Self>> {
        let id = Uuid::parse_str(id)?;
        let pool = get_pool();
        let row = sqlx::query_as::<_, Self>(
            "SELECT id, title, slug, excerpt, content, tags, published, reading_time, created_at, updated_at FROM posts WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;
        Ok(row)
    }

    pub async fn recent(limit: usize) -> AppResult<Vec<Self>> {
        let pool = get_pool();
        let limit = limit as i64;
        let rows = sqlx::query_as::<_, Self>(
            "SELECT id, title, slug, excerpt, content, tags, published, reading_time, created_at, updated_at FROM posts WHERE published = true ORDER BY created_at DESC LIMIT $1",
        )
        .bind(limit)
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    pub async fn create(data: CreatePost) -> AppResult<Self> {
        let slug = slug::slugify(&data.title);
        let reading_time = Self::calculate_reading_time(&data.content);
        let pool = get_pool();

        let row = sqlx::query_as::<_, Self>(
            r#"
            INSERT INTO posts (title, slug, excerpt, content, tags, published, reading_time)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, title, slug, excerpt, content, tags, published, reading_time, created_at, updated_at
            "#,
        )
        .bind(&data.title)
        .bind(&slug)
        .bind(&data.excerpt)
        .bind(&data.content)
        .bind(&data.tags)
        .bind(data.published)
        .bind(reading_time)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    pub async fn update(id: &str, data: UpdatePost) -> AppResult<Self> {
        let id = Uuid::parse_str(id)?;
        let pool = get_pool();

        let current = Self::by_id(&id.to_string())
            .await?
            .ok_or_else(|| AppError::NotFound("Post not found".to_string()))?;

        let title = data.title.as_deref().unwrap_or(&current.title);
        let slug = slug::slugify(title);
        let excerpt = data.excerpt.as_deref().unwrap_or(&current.excerpt);
        let content = data.content.as_deref().unwrap_or(&current.content);
        let tags = data.tags.as_ref().unwrap_or(&current.tags);
        let published = data.published.unwrap_or(current.published);
        let reading_time = data
            .content
            .as_ref()
            .map(|c| Self::calculate_reading_time(c))
            .unwrap_or(current.reading_time);

        let row = sqlx::query_as::<_, Self>(
            r#"
            UPDATE posts SET title = $1, slug = $2, excerpt = $3, content = $4, tags = $5, published = $6, reading_time = $7, updated_at = now()
            WHERE id = $8
            RETURNING id, title, slug, excerpt, content, tags, published, reading_time, created_at, updated_at
            "#,
        )
        .bind(title)
        .bind(slug)
        .bind(excerpt)
        .bind(content)
        .bind(tags)
        .bind(published)
        .bind(reading_time)
        .bind(id)
        .fetch_optional(pool)
        .await?;

        row.ok_or_else(|| AppError::NotFound("Post not found".to_string()))
    }

    pub async fn delete(id: &str) -> AppResult<()> {
        let id = Uuid::parse_str(id)?;
        let pool = get_pool();
        let result = sqlx::query("DELETE FROM posts WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Post not found".to_string()));
        }
        Ok(())
    }

    pub async fn count() -> AppResult<i64> {
        let pool = get_pool();
        let row: (i64,) = sqlx::query_as("SELECT count(*) FROM posts")
            .fetch_one(pool)
            .await?;
        Ok(row.0)
    }

    pub async fn count_published() -> AppResult<i64> {
        let pool = get_pool();
        let row: (i64,) = sqlx::query_as("SELECT count(*) FROM posts WHERE published = true")
            .fetch_one(pool)
            .await?;
        Ok(row.0)
    }
}
