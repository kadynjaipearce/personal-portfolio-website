use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Experience {
    pub id: Option<Uuid>,
    pub company: String,
    pub role: String,
    pub description: String,
    pub start_date: String,
    pub end_date: Option<String>,
    pub current: bool,
    pub order_index: i32,
}

#[derive(Debug, Deserialize)]
pub struct CreateExperience {
    pub company: String,
    pub role: String,
    pub description: String,
    pub start_date: String,
    pub end_date: Option<String>,
    pub current: bool,
    pub order_index: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateExperience {
    pub company: Option<String>,
    pub role: Option<String>,
    pub description: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub current: Option<bool>,
    pub order_index: Option<i32>,
}

impl Experience {
    pub async fn all() -> AppResult<Vec<Self>> {
        let pool = get_pool();
        let rows = sqlx::query_as::<_, Self>(
            "SELECT id, company, role, description, start_date, end_date, current, order_index FROM experiences ORDER BY order_index ASC",
        )
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    pub async fn by_id(id: &str) -> AppResult<Option<Self>> {
        let id = Uuid::parse_str(id)?;
        let pool = get_pool();
        let row = sqlx::query_as::<_, Self>(
            "SELECT id, company, role, description, start_date, end_date, current, order_index FROM experiences WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;
        Ok(row)
    }

    pub async fn create(data: CreateExperience) -> AppResult<Self> {
        let pool = get_pool();

        let row = sqlx::query_as::<_, Self>(
            r#"
            INSERT INTO experiences (company, role, description, start_date, end_date, current, order_index)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, company, role, description, start_date, end_date, current, order_index
            "#,
        )
        .bind(&data.company)
        .bind(&data.role)
        .bind(&data.description)
        .bind(&data.start_date)
        .bind(&data.end_date)
        .bind(data.current)
        .bind(data.order_index)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    pub async fn update(id: &str, data: UpdateExperience) -> AppResult<Self> {
        let id = Uuid::parse_str(id)?;
        let pool = get_pool();

        let current = Self::by_id(&id.to_string())
            .await?
            .ok_or_else(|| AppError::NotFound("Experience not found".to_string()))?;

        let company = data.company.as_deref().unwrap_or(&current.company);
        let role = data.role.as_deref().unwrap_or(&current.role);
        let description = data.description.as_deref().unwrap_or(&current.description);
        let start_date = data.start_date.as_deref().unwrap_or(&current.start_date);
        let end_date = data.end_date.as_deref().or(current.end_date.as_deref());
        let current_flag = data.current.unwrap_or(current.current);
        let order_index = data.order_index.unwrap_or(current.order_index);

        let row = sqlx::query_as::<_, Self>(
            r#"
            UPDATE experiences SET company = $1, role = $2, description = $3, start_date = $4, end_date = $5, current = $6, order_index = $7
            WHERE id = $8
            RETURNING id, company, role, description, start_date, end_date, current, order_index
            "#,
        )
        .bind(company)
        .bind(role)
        .bind(description)
        .bind(start_date)
        .bind(end_date)
        .bind(current_flag)
        .bind(order_index)
        .bind(id)
        .fetch_optional(pool)
        .await?;

        row.ok_or_else(|| AppError::NotFound("Experience not found".to_string()))
    }

    pub async fn delete(id: &str) -> AppResult<()> {
        let id = Uuid::parse_str(id)?;
        let pool = get_pool();
        let result = sqlx::query("DELETE FROM experiences WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Experience not found".to_string()));
        }
        Ok(())
    }
}
