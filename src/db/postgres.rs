use crate::config::CONFIG;
use crate::error::AppError;
use once_cell::sync::OnceCell;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tracing::info;

static POOL: OnceCell<PgPool> = OnceCell::new();

pub async fn init_db() -> Result<(), AppError> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&CONFIG.database_url)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    POOL.set(pool)
        .map_err(|_| AppError::DatabaseError("Database already initialized".to_string()))?;

    seed_data().await?;

    info!("Database initialized successfully");
    Ok(())
}

pub fn get_pool() -> &'static PgPool {
    POOL.get().expect("Database not initialized")
}

async fn seed_data() -> Result<(), AppError> {
    let pool = get_pool();

    let exists: (bool,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM projects LIMIT 1)")
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if exists.0 {
        info!("Database already seeded, skipping");
        return Ok(());
    }

    info!("Seeding database with empty content - add via admin dashboard");
    info!("Database seeded successfully");
    Ok(())
}
