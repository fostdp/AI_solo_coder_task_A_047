use sqlx::{PgPool, postgres::PgPoolOptions};

pub async fn init_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(20)
        .connect(database_url)
        .await
}

pub type DbPool = PgPool;
