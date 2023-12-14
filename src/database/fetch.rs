use std::{collections::HashSet, ops::DerefMut};

use eyre::{Context as _, Report, Result};
use futures_util::{StreamExt, TryStreamExt};

use crate::{
    model::{MedalRarities, SlimBadge},
    util::IntHasher,
};

use super::Database;

impl Database {
    pub async fn fetch_osekai_ranking_ids(
        &self,
        user_ids: &mut HashSet<u32, IntHasher>,
    ) -> Result<()> {
        let mut conn = self
            .acquire()
            .await
            .context("failed to acquire connection to fetch ranking ids")?;

        let query = sqlx::query!(
            r#"
SELECT 
  id 
FROM 
  Ranking"#
        );

        query
            .fetch(conn.deref_mut())
            .try_for_each(|row| {
                user_ids.insert(row.id as u32);

                std::future::ready(Ok(()))
            })
            .await
            .context("failed to fetch all ranking ids")?;

        Ok(())
    }

    pub async fn fetch_osekai_user_ids(&self) -> Result<HashSet<u32, IntHasher>> {
        let mut conn = self
            .acquire()
            .await
            .context("failed to acquire connection to fetch user ids")?;

        let query = sqlx::query!(
            r#"
SELECT 
  id 
FROM 
  Members"#
        );

        query
            .fetch(conn.deref_mut())
            .map_ok(|row| row.id as u32)
            .try_collect()
            .await
            .context("failed to fetch all user ids")
    }

    /// The resulting badges will be sorted by their description.
    pub async fn fetch_badges(&self) -> Result<Vec<SlimBadge>> {
        let mut conn = self
            .acquire()
            .await
            .context("failed to acquire connection to fetch badges")?;

        let query = sqlx::query!(
            r#"
SELECT 
  id, 
  description, 
  image 
FROM 
  badges"#
        );

        let mut badges: Vec<_> = query
            .fetch(conn.deref_mut())
            .map(|res| {
                let row = res?;

                Ok::<_, Report>(SlimBadge {
                    id: row.id as u32,
                    description: row.description.into_boxed_str(),
                    image_url: row.image.into_boxed_str(),
                })
            })
            .try_collect()
            .await
            .context("failed to fetch all badges")?;

        badges.sort_unstable();

        Ok(badges)
    }

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
