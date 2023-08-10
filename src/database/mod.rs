mod fetch;
mod store;

use eyre::{Context as _, Result};
use sqlx::{pool::PoolConnection, Error as SqlxError, MySql, MySqlPool, Transaction};

pub struct Database {
    mysql: MySqlPool,
}

impl Database {
    pub async fn new(url: &str) -> Result<Self> {
        MySqlPool::connect(url)
            .await
            .map(|mysql| Self { mysql })
            .context("failed to connect to database")
    }

    async fn acquire(&self) -> Result<PoolConnection<MySql>, SqlxError> {
        self.mysql.acquire().await
    }

    async fn begin(&self) -> Result<Transaction<'_, MySql>, SqlxError> {
        self.mysql.begin().await
    }
}
