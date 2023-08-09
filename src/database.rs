use std::ops::DerefMut;

use eyre::{Context as _, Result};
use sqlx::{pool::PoolConnection, Error as SqlxError, MySql, MySqlPool, Transaction};

use crate::model::{BadgeEntry, BadgeKey, Badges, MedalRarities, MedalRarityEntry, ScrapedMedal};

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

    async fn begin(&self) -> Result<Transaction<'_, MySql>, SqlxError> {
        self.mysql.begin().await
    }

    pub async fn store_medals(&self, medals: &[ScrapedMedal]) -> Result<()> {
        let mut tx = self
            .begin()
            .await
            .context("failed to begin transaction for Medals")?;

        for medal in medals {
            let ScrapedMedal {
                icon_url,
                id,
                name,
                grouping,
                ordering,
                slug: _,
                description,
                mode,
                instructions,
            } = medal;

            let query = sqlx::query!(
                r#"
INSERT INTO Medals (
  medalid, name, link, description, 
  restriction, `grouping`, instructions, 
  ordering
) 
VALUES 
  (?, ?, ?, ?, ?, ?, ?, ?) ON DUPLICATE KEY 
UPDATE 
  medalid = VALUES(medalid), 
  name = VALUES(name), 
  link = VALUES(link), 
  description = VALUES(description), 
  restriction = VALUES(restriction), 
  `grouping` = VALUES(`grouping`), 
  ordering = VALUES(ordering), 
  instructions = VALUES(instructions)"#,
                id,
                name.as_ref(),
                icon_url.as_ref(),
                description.as_ref(),
                mode.as_deref(),
                grouping.as_ref(),
                ordering,
                instructions.as_deref(),
            );

            query
                .execute(tx.deref_mut())
                .await
                .context("failed to execute Medals query")?;
        }

        tx.commit()
            .await
            .context("failed to commit Medals transaction")?;

        Ok(())
    }

    pub async fn store_rarities(&self, rarities: &MedalRarities) -> Result<()> {
        let mut tx = self
            .begin()
            .await
            .context("failed to begin transaction for MedalRarity")?;

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
        let mut tx = self
            .begin()
            .await
            .context("failed to begin transaction for Badges")?;

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
