use std::ops::DerefMut;

use eyre::{Context as _, Result};
use sqlx::{pool::PoolConnection, MySql, MySqlPool, Transaction};

use crate::model::{BadgeEntry, BadgeKey, Badges, MedalRarities, MedalRarityEntry};

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

    async fn acquire(&self) -> Result<PoolConnection<MySql>> {
        self.mysql
            .acquire()
            .await
            .context("failed to acquire database connection")
    }

    async fn begin(&self) -> Result<Transaction<'_, MySql>> {
        self.mysql
            .begin()
            .await
            .context("failed to begin database transaction")
    }

    pub async fn store_rarities(&self, rarities: &MedalRarities) -> Result<()> {
        let mut tx = self.begin().await?;

        for (medal_id, MedalRarityEntry { count, frequency }) in rarities.iter() {
            let query = sqlx::query!(
                r#"
INSERT INTO MedalRarity (id, frequency, count) 
VALUES 
  (?, ?, ?) ON DUPLICATE KEY 
UPDATE 
  id = VALUES(id), 
  frequency = VALUES(frequency), 
  count = VALUES(count)"#,
                medal_id,
                frequency,
                count
            );

            query
                .execute(tx.deref_mut())
                .await
                .context("failed to execute MedalRarity query")?;
        }

        tx.commit()
            .await
            .context("failed to commit MedalRarity transaction")?;

        Ok(())
    }

    pub async fn store_badges(&self, badges: &Badges) -> Result<()> {
        let mut tx = self.begin().await?;

        sqlx::query!("DELETE FROM Badges")
            .execute(tx.deref_mut())
            .await
            .context("failed to delete rows in Badges")?;

        for (key, value) in badges.iter() {
            let BadgeKey { image_url } = key;

            let BadgeEntry {
                description,
                id,
                awarded_at,
                users,
            } = value;

            let name = image_url
                .rsplit_once('/')
                .and_then(|(_, file)| file.rsplit_once('.'))
                .map(|(name, _)| name.replace(['-', '_'], " "));

            let query = sqlx::query!(
                r#"
INSERT INTO Badges (
  id, name, image_url, description, awarded_at, users
) 
VALUES 
  (?, ?, ?, ?, ?, ?)"#,
                id,
                name,
                image_url.as_ref(),
                description.as_ref(),
                awarded_at,
                users.to_string(),
            );

            query
                .execute(tx.deref_mut())
                .await
                .context("failed to execute Badges query")?;
        }

        tx.commit()
            .await
            .context("failed to commit Badges transaction")?;

        Ok(())
    }
}
