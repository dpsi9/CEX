use shared::CexError;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tracing::info;

pub struct Db {
    pool: PgPool,
}

impl Db {
    pub async fn new(url: &str, max_connections: u32) -> Result<Self, CexError> {
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .connect(url)
            .await
            .map_err(|e| CexError::Internal(format!("db connection error: {e}")))?;
        info!("db connected");
        Ok(Self { pool })
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

pub async fn insert_event(pool: &PgPool, payload: &str) -> Result<(), CexError> {
    sqlx::query("INSERT INTO events (payload) VALUES ($1)")
        .bind(payload)
        .execute(pool)
        .await
        .map_err(|e| CexError::Internal(format!("insert event failed: {e}")))?
        .rows_affected();
    Ok(())
}

/// Run embedded SQL migrations from `db/migrations`.
pub async fn migrate(pool: &PgPool) -> Result<(), CexError> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|e| CexError::Internal(format!("migration failed: {e}")))?;
    Ok(())
}
