use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Project {
    pub id: Option<Uuid>,
    pub name: String,
    pub company: Option<String>,
    pub description: String,
    pub project_type: String,
    pub url: Option<String>,
    pub site_image_url: Option<String>,
    pub client_image_url: Option<String>,
    pub tags: Vec<String>,
    pub stars: i16,
    pub sort_order: i16,
    pub is_featured: bool,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProject {
    pub name: String,
    pub company: Option<String>,
    pub description: String,
    pub project_type: String,
    pub url: Option<String>,
    pub site_image_url: Option<String>,
    pub client_image_url: Option<String>,
    pub tags: Vec<String>,
    pub stars: i16,
    pub sort_order: i16,
    pub is_featured: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProject {
    pub name: Option<String>,
    pub company: Option<String>,
    pub description: Option<String>,
    pub project_type: Option<String>,
    pub url: Option<String>,
    pub site_image_url: Option<String>,
    pub client_image_url: Option<String>,
    pub tags: Option<Vec<String>>,
    pub stars: Option<i16>,
    pub sort_order: Option<i16>,
    pub is_featured: Option<bool>,
}

impl Project {
    pub async fn all() -> AppResult<Vec<Self>> {
        let pool = get_pool();
        let rows = sqlx::query_as::<_, Self>(
            "SELECT id, name, company, description, project_type, url, site_image_url, client_image_url, tags, stars, sort_order, is_featured, created_at FROM projects ORDER BY sort_order ASC, created_at DESC",
        )
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    pub async fn featured() -> AppResult<Vec<Self>> {
        let pool = get_pool();
        let rows = sqlx::query_as::<_, Self>(
            "SELECT id, name, company, description, project_type, url, site_image_url, client_image_url, tags, stars, sort_order, is_featured, created_at FROM projects WHERE is_featured = true ORDER BY sort_order ASC, created_at DESC LIMIT 3",
        )
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    pub async fn by_project_type(project_type: &str) -> AppResult<Vec<Self>> {
        let pool = get_pool();
        let rows = sqlx::query_as::<_, Self>(
            "SELECT id, name, company, description, project_type, url, site_image_url, client_image_url, tags, stars, sort_order, is_featured, created_at FROM projects WHERE project_type = $1 ORDER BY sort_order ASC, created_at DESC",
        )
        .bind(project_type)
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    pub async fn by_id(id: &str) -> AppResult<Option<Self>> {
        let id = Uuid::parse_str(id)?;
        let pool = get_pool();
        let row = sqlx::query_as::<_, Self>(
            "SELECT id, name, company, description, project_type, url, site_image_url, client_image_url, tags, stars, sort_order, is_featured, created_at FROM projects WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;
        Ok(row)
    }

    pub async fn create(data: CreateProject) -> AppResult<Self> {
        let pool = get_pool();

        let row = sqlx::query_as::<_, Self>(
            r#"
            INSERT INTO projects (name, company, description, project_type, url, site_image_url, client_image_url, tags, stars, sort_order, is_featured)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id, name, company, description, project_type, url, site_image_url, client_image_url, tags, stars, sort_order, is_featured, created_at
            "#,
        )
        .bind(&data.name)
        .bind(&data.company)
        .bind(&data.description)
        .bind(&data.project_type)
        .bind(&data.url)
        .bind(&data.site_image_url)
        .bind(&data.client_image_url)
        .bind(&data.tags)
        .bind(data.stars)
        .bind(data.sort_order)
        .bind(data.is_featured)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    pub async fn update(id: &str, data: UpdateProject) -> AppResult<Self> {
        let id = Uuid::parse_str(id)?;
        let pool = get_pool();

        let current = Self::by_id(&id.to_string())
            .await?
            .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

        let name = data.name.as_deref().unwrap_or(&current.name);
        let company = data.company.as_deref().or(current.company.as_deref());
        let description = data.description.as_deref().unwrap_or(&current.description);
        let project_type = data
            .project_type
            .as_deref()
            .unwrap_or(&current.project_type);
        let url = data.url.as_deref().or(current.url.as_deref());
        let site_image_url = data
            .site_image_url
            .as_deref()
            .or(current.site_image_url.as_deref());
        let client_image_url = data
            .client_image_url
            .as_deref()
            .or(current.client_image_url.as_deref());
        let tags = data.tags.as_ref().unwrap_or(&current.tags);
        let stars = data.stars.unwrap_or(current.stars);
        let sort_order = data.sort_order.unwrap_or(current.sort_order);
        let is_featured = data.is_featured.unwrap_or(current.is_featured);

        let row = sqlx::query_as::<_, Self>(
            r#"
            UPDATE projects SET name = $1, company = $2, description = $3, project_type = $4, url = $5, site_image_url = $6, client_image_url = $7, tags = $8, stars = $9, sort_order = $10, is_featured = $11
            WHERE id = $12
            RETURNING id, name, company, description, project_type, url, site_image_url, client_image_url, tags, stars, sort_order, is_featured, created_at
            "#,
        )
        .bind(name)
        .bind(company)
        .bind(description)
        .bind(project_type)
        .bind(url)
        .bind(site_image_url)
        .bind(client_image_url)
        .bind(tags)
        .bind(stars)
        .bind(sort_order)
        .bind(is_featured)
        .bind(id)
        .fetch_optional(pool)
        .await?;

        row.ok_or_else(|| AppError::NotFound("Project not found".to_string()))
    }

    pub async fn delete(id: &str) -> AppResult<()> {
        let id = Uuid::parse_str(id)?;
        let pool = get_pool();
        let result = sqlx::query("DELETE FROM projects WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Project not found".to_string()));
        }
        Ok(())
    }

    pub async fn count() -> AppResult<i64> {
        let pool = get_pool();
        let row: (i64,) = sqlx::query_as("SELECT count(*) FROM projects")
            .fetch_one(pool)
            .await?;
        Ok(row.0)
    }
}
