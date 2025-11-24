use sqlx::{PgPool, postgres::PgPoolOptions};
use std::ops::Deref;

pub mod member;
pub mod team;
pub mod user;

#[derive(Debug)]
pub struct Repo {
    pool: PgPool,
}

impl Repo {
    const MAX_CONNECTIONS: u32 = 5;

    pub async fn new() -> anyhow::Result<Self> {
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|err| anyhow::anyhow!("Error when acquiring DATABASE_URL: {}", err))?;

        let pool = PgPoolOptions::new()
            .max_connections(Self::MAX_CONNECTIONS)
            .connect(&database_url)
            .await
            .map_err(|err| anyhow::anyhow!("Error when connecting to Database: {}", err))?;

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn ping(&self) -> anyhow::Result<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(|err| anyhow::anyhow!("Error when PING Database: {}", err))?;

        Ok(())
    }
}
