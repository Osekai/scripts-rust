use std::{collections::HashSet, ops::DerefMut};

use eyre::{Context as _, Result};
use futures_util::TryStreamExt;

use crate::{model::MedalRarities, util::IntHasher};

use super::Database;

impl Database {
    pub async fn fetch_medal_rarities(&self) -> Result<MedalRarities> {
        let mut conn = self
            .acquire()
            .await
            .context("failed to acquire connection to fetch medal rarities")?;

        let query = sqlx::query!(
            r#"
SELECT 
  id, 
  frequency, 
  count 
FROM 
  MedalRarity"#
        );

        query
            .fetch(conn.deref_mut())
            .map_ok(|row| (row.id as u16, row.count as u32, row.frequency))
            .try_collect()
            .await
            .context("failed to fetch all medal rarities")
    }

    pub async fn fetch_medal_ids(&self) -> Result<HashSet<u16, IntHasher>> {
        let mut conn = self
            .acquire()
            .await
            .context("failed to acquire connection to fetch medal ids")?;

        let query = sqlx::query!(
            r#"
SELECT 
  medalid 
FROM 
  Medals"#
        );

        query
            .fetch(conn.deref_mut())
            .map_ok(|row| row.medalid as u16)
            .try_collect()
            .await
            .context("failed to fetch all medal ids")
    }
}
