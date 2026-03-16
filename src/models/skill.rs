use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Skill {
    pub id: Option<Uuid>,
    pub name: String,
    pub category: String,
    pub icon: Option<String>,
    pub proficiency: i32,
}

#[derive(Debug, Deserialize)]
pub struct CreateSkill {
    pub name: String,
    pub category: String,
    pub icon: Option<String>,
    pub proficiency: i32,
}

impl Skill {
    pub async fn all() -> AppResult<Vec<Self>> {
        let pool = get_pool();
        let rows = sqlx::query_as::<_, Self>(
            "SELECT id, name, category, icon, proficiency FROM skills ORDER BY category, proficiency DESC",
        )
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    pub async fn grouped() -> AppResult<std::collections::HashMap<String, Vec<Self>>> {
        let all = Self::all().await?;
        let mut grouped: std::collections::HashMap<String, Vec<Self>> =
            std::collections::HashMap::new();

        for skill in all {
            grouped
                .entry(skill.category.clone())
                .or_default()
                .push(skill);
        }

        Ok(grouped)
    }

    pub async fn create(data: CreateSkill) -> AppResult<Self> {
        let pool = get_pool();

        let row = sqlx::query_as::<_, Self>(
            r#"
            INSERT INTO skills (name, category, icon, proficiency)
            VALUES ($1, $2, $3, $4)
            RETURNING id, name, category, icon, proficiency
            "#,
        )
        .bind(&data.name)
        .bind(&data.category)
        .bind(&data.icon)
        .bind(data.proficiency)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    pub async fn delete(id: &str) -> AppResult<()> {
        let id = Uuid::parse_str(id)?;
        let pool = get_pool();
        let result = sqlx::query("DELETE FROM skills WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Skill not found".to_string()));
        }
        Ok(())
    }
}
