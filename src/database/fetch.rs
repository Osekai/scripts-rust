use std::{
    collections::{HashMap, HashSet},
    ops::DerefMut,
};

use eyre::{Context as _, Result};
use futures_util::{future, TryStreamExt};
use time::OffsetDateTime;

use crate::{
    model::{BadgeDescription, BadgeImageUrl, BadgeName, BadgeOwner, Badges, MedalRarities},
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

        let query = sqlx::query!(r#"SELECT `ID` as id FROM Rankings_Users"#);

        query
            .fetch(conn.deref_mut())
            .try_for_each(|row| {
                user_ids.insert(row.id as u32);

                std::future::ready(Ok(()))
            })
            .await
            .context("failed to fetch Rankings_Users ids")?;

        let query = sqlx::query!(
            r#"
            SELECT `First_Achieved_User_ID` as id
            FROM Medals_Configuration
            WHERE `First_Achieved_User_ID` IS NOT NULL"#
        );

        query
            .fetch(conn.deref_mut())
            .try_for_each(|row| {
                user_ids.insert(row.id.unwrap_or(0) as u32);

                std::future::ready(Ok(()))
            })
            .await
            .context("failed to fetch Medals_Configuration ids")?;

        Ok(())
    }

    pub async fn fetch_osekai_user_ids(&self) -> Result<HashSet<u32, IntHasher>> {
        let mut conn = self
            .acquire()
            .await
            .context("failed to acquire connection to fetch system user ids")?;

        let query = sqlx::query!(
            r#"
        SELECT
          `User_ID` as id
        FROM
          System_Users"#
        );

        query
            .fetch(conn.deref_mut())
            .map_ok(|row| row.id as u32)
            .try_collect()
            .await
            .context("failed to fetch system user ids")
    }

    pub async fn fetch_badges(&self) -> Result<Badges> {
        let mut conn = self
            .acquire()
            .await
            .context("failed to acquire connection to fetch badges")?;

        let query =
            sqlx::query!("SELECT `Name` AS name, `Image_URL` AS image_url FROM `Badges_Data`");

        let names_fut = query
            .fetch(conn.deref_mut())
            .map_ok(|row| {
                let name = row.name.into_boxed_str();
                let image_url = row.image_url.into_boxed_str();

                (BadgeName(name), BadgeImageUrl(image_url))
            })
            .try_collect();

        let mut stored = Badges {
            names: names_fut.await.context("failed to fetch badges data")?,
            descriptions: HashMap::default(),
        };

        let query = sqlx::query!(
            r#"
SELECT user_id, description, awarded_at, name
FROM
    (SELECT
        `Badge_ID` AS badge_id, 
        `User_ID` AS user_id, 
        `Description` AS description, 
        `Date_Awarded` AS awarded_at 
    FROM `Badges_Users`) AS u
    JOIN
    (SELECT `ID` AS badge_id, `Name` AS name FROM `Badges_Data`) AS n
    USING (badge_id)"#
        );

        query
            .fetch(conn.deref_mut())
            .try_for_each(|row| {
                let owner = BadgeOwner {
                    user_id: row.user_id as u32,
                    awarded_at: row
                        .awarded_at
                        .map(|datetime| datetime.assume_utc())
                        .unwrap_or_else(OffsetDateTime::now_utc),
                };

                let description = row.description.unwrap_or_default();

                if let Some(names) = stored.descriptions.get_mut(description.as_str()) {
                    if let Some(owners) = names.get_mut(row.name.as_str()) {
                        owners.insert(owner);
                    } else {
                        names
                            .entry(BadgeName(row.name.into_boxed_str()))
                            .or_default()
                            .insert(owner);
                    }
                } else {
                    let mut owners = HashSet::default();
                    owners.insert(owner);
                    let mut names = HashMap::default();
                    names.insert(BadgeName(row.name.into_boxed_str()), owners);
                    let description = BadgeDescription(description.into_boxed_str());
                    stored.descriptions.insert(description, names);
                }

                future::ready(Ok(()))
            })
            .await
            .context("failed to fetch badges users")?;

        Ok(stored)
    }

    pub async fn fetch_medal_rarities(&self) -> Result<MedalRarities> {
        let mut conn = self
            .acquire()
            .await
            .context("failed to acquire connection to fetch medal rarities")?;

        let query = sqlx::query!(
            r#"
        SELECT
          `Medal_ID` as id,
          `Frequency` as frequency,
          `Count_Achieved_By` as count
        FROM
          Medals_Data"#
        );

        query
            .fetch(conn.deref_mut())
            .map_ok(|row| {
                (
                    row.id as u16,
                    row.count.unwrap_or(0) as u32,
                    row.frequency.unwrap_or(0.0),
                )
            })
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
          `Medal_ID` as id
        FROM
          Medals_Data"#
        );

        query
            .fetch(conn.deref_mut())
            .map_ok(|row| row.id as u16)
            .try_collect()
            .await
            .context("failed to fetch all medal ids")
    }
}
