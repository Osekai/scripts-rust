use std::{num::NonZeroU32, ops::DerefMut};

use eyre::{Context as _, Result};
use tokio::task::JoinHandle;

use crate::model::{
    BadgeDescription, BadgeImageUrl, BadgeName, BadgeOwner, Badges, Finish, MedalRarities,
    MedalRarityEntry, Progress, RankingUser, RankingsIter, ScrapedMedal,
};

use super::Database;

impl Database {
    pub async fn store_progress(&self, progress: &Progress) -> Result<()> {
        let mut conn = self
            .acquire()
            .await
            .context("failed to acquire connection to upsert Rankings_Script_History")?;

        let Progress {
            start,
            current,
            total,
            eta_seconds,
            task,
        } = progress;

        let query = sqlx::query!(
            r#"
INSERT INTO 
  Rankings_Script_History (
    `ID`, 
    `Type`, 
    `Time`, 
    `Count_Current`, 
    `Count_Total`, 
    `Elapsed_Seconds`, 
    `Elapsed_Last_Update` 
) VALUES (?, ?, ?, ?, ?, ?, NOW())
ON DUPLICATE KEY UPDATE
  `Count_Current` = VALUES(`Count_Current`), 
  `Count_Total` = VALUES(`Count_Total`), 
  `Elapsed_Seconds` = VALUES(`Elapsed_Seconds`), 
  `Elapsed_Last_Update` = VALUES(`Elapsed_Last_Update`)"#,
            start.unix_timestamp(),
            task.to_string(),
            start,
            *current as i32,
            *total as i32,
            eta_seconds,
        );

        query
            .execute(conn.deref_mut())
            .await
            .context("failed to execute Rankings_Script_History query")?;

        Ok(())
    }

    pub async fn store_finish(&self, finish: &Finish) -> Result<()> {
        let mut conn = self
            .acquire()
            .await
            .context("failed to acquire connection to finish Rankings_Script_History")?;

        let Finish {
            id,
            requested_users,
        } = finish;

        let query = sqlx::query!(
            r#"
UPDATE 
  Rankings_Script_History
SET 
  `Count_Current` = ?, 
  `Count_Total` = ?, 
  `Elapsed_Seconds` = ?, 
  `Elapsed_Last_Update` = NOW()
WHERE
  `ID` = ?"#,
            *requested_users as i64,
            *requested_users as i64,
            0,
            id,
        );

        query
            .execute(conn.deref_mut())
            .await
            .context("failed to execute Rankings_Script_History query")?;

        Ok(())
    }

    #[must_use]
    pub fn store_rankings(&self, rankings: RankingsIter) -> JoinHandle<()> {
        async fn inner(db: Database, rankings: RankingsIter) -> Result<()> {
            let mut tx = db
                .begin()
                .await
                .context("failed to begin transaction for Rankings_Users")?;

            for ranking in rankings {
                let stdev_acc = ranking.std_dev_acc();
                let stdev_level = ranking.std_dev_level();
                let stdev_pp = ranking.std_dev_pp();
                let total_pp = ranking.total_pp();

                let RankingUser {
                    id,
                    name,
                    ignore_acc,
                    medal_count,
                    rarest_medal_id,
                    rarest_medal_achieved,
                    country_code,
                    badge_count,
                    ranked_maps,
                    loved_maps,
                    subscribers,
                    replays_watched,
                    restricted,
                    std,
                    tko,
                    ctb,
                    mna,
                } = ranking;

                let mut std_acc = std.acc;
                let mut tko_acc = tko.acc;
                let mut ctb_acc = ctb.acc;
                let mut mna_acc = mna.acc;

                if ignore_acc {
                    std_acc = 0.0;
                    tko_acc = 0.0;
                    ctb_acc = 0.0;
                    mna_acc = 0.0;
                }

                let query = sqlx::query!(
                    r#"
            INSERT INTO Rankings_Users (
                `ID`, `Accuracy_Catch`, `Accuracy_Mania`, `Accuracy_Standard`, 
                `Accuracy_Stdev`, `Accuracy_Taiko`, `Count_Badges`, 
                `Count_Maps_Loved`, `Count_Maps_Ranked`, `Count_Medals`, 
                `Count_Replays_Watched`, `Count_Subscribers`, `Country_Code`, 
                `Is_Restricted`, `Level_Catch`, `Level_Mania`, `Level_Standard`, 
                `Level_Stdev`, `Level_Taiko`, `Name`, `PP_Catch`, `PP_Mania`, 
                `PP_Standard`, `PP_Stdev`, `PP_Taiko`, `PP_Total`, 
                `Rank_Global_Catch`, `Rank_Global_Mania`, `Rank_Global_Standard`, 
                `Rank_Global_Taiko`, `Rarest_Medal_Achieved`, `Rarest_Medal_ID`
            )
            VALUES
              (
                ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?,
                ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
              ) ON DUPLICATE KEY
            UPDATE
                `ID` = VALUES(`ID`), 
                `Accuracy_Catch` = VALUES(`Accuracy_Catch`), 
                `Accuracy_Mania` = VALUES(`Accuracy_Mania`), 
                `Accuracy_Standard` = VALUES(`Accuracy_Standard`), 
                `Accuracy_Stdev` = VALUES(`Accuracy_Stdev`), 
                `Accuracy_Taiko` = VALUES(`Accuracy_Taiko`), 
                `Count_Badges` = VALUES(`Count_Badges`), 
                `Count_Maps_Loved` = VALUES(`Count_Maps_Loved`), 
                `Count_Maps_Ranked` = VALUES(`Count_Maps_Ranked`), 
                `Count_Medals` = VALUES(`Count_Medals`), 
                `Count_Replays_Watched` = VALUES(`Count_Replays_Watched`), 
                `Count_Subscribers` = VALUES(`Count_Subscribers`), 
                `Country_Code` = VALUES(`Country_Code`), 
                `Is_Restricted` = VALUES(`Is_Restricted`), 
                `Level_Catch` = VALUES(`Level_Catch`), 
                `Level_Mania` = VALUES(`Level_Mania`), 
                `Level_Standard` = VALUES(`Level_Standard`), 
                `Level_Stdev` = VALUES(`Level_Stdev`), 
                `Level_Taiko` = VALUES(`Level_Taiko`), 
                `Name` = VALUES(`Name`), 
                `PP_Catch` = VALUES(`PP_Catch`), 
                `PP_Mania` = VALUES(`PP_Mania`), 
                `PP_Standard` = VALUES(`PP_Standard`), 
                `PP_Stdev` = VALUES(`PP_Stdev`), 
                `PP_Taiko` = VALUES(`PP_Taiko`), 
                `PP_Total` = VALUES(`PP_Total`), 
                `Rank_Global_Catch` = VALUES(`Rank_Global_Catch`), 
                `Rank_Global_Mania` = VALUES(`Rank_Global_Mania`), 
                `Rank_Global_Standard` = VALUES(`Rank_Global_Standard`), 
                `Rank_Global_Taiko` = VALUES(`Rank_Global_Taiko`), 
                `Rarest_Medal_Achieved` = VALUES(`Rarest_Medal_Achieved`), 
                `Rarest_Medal_ID` = VALUES(`Rarest_Medal_ID`)"#,
                    id,
                    ctb_acc,
                    mna_acc,
                    std_acc,
                    stdev_acc,
                    tko_acc,
                    badge_count,
                    loved_maps,
                    ranked_maps,
                    medal_count,
                    replays_watched,
                    subscribers,
                    country_code.as_ref(),
                    restricted as u8,
                    ctb.level,
                    mna.level,
                    std.level,
                    stdev_level,
                    tko.level,
                    name.as_ref(),
                    ctb.pp,
                    mna.pp,
                    std.pp,
                    stdev_pp,
                    tko.pp,
                    total_pp,
                    ctb.global_rank.map(NonZeroU32::get),
                    mna.global_rank.map(NonZeroU32::get),
                    std.global_rank.map(NonZeroU32::get),
                    tko.global_rank.map(NonZeroU32::get),
                    rarest_medal_achieved,
                    rarest_medal_id,
                );

                query
                    .execute(tx.deref_mut())
                    .await
                    .context("failed to execute Rankings_Users query")?;
            }

            tx.commit()
                .await
                .context("failed to commit Rankings_Users transaction")?;

            Ok(())
        }

        let db = self.to_owned();

        tokio::spawn(async move {
            let len = rankings.len();
            let res = inner(db, rankings).await;
            let _entered = info_span!("store_rankings").entered();

            match res {
                Ok(_) => info!("Successfully stored {len} ranking entries"),
                Err(err) => error!(?err, "Failed to store rankings"),
            }
        })
    }

    // This method does not return a JoinHandle but is async instead and should
    // be called before `Database::store_rarities` so that the table does not
    // deadlock.
    pub async fn store_medals(&self, medals: &[ScrapedMedal]) {
        async fn inner(db: &Database, medals: &[ScrapedMedal]) -> Result<()> {
            let mut tx = db
                .begin()
                .await
                .context("failed to begin transaction for Medals_Data")?;

            for medal in medals {
                let ScrapedMedal {
                    icon_url,
                    id,
                    name,
                    grouping,
                    ordering,
                    description,
                    mode,
                    instructions,
                } = medal;

                let link = icon_url.rsplit('/').next().unwrap_or(&icon_url);

                let query = sqlx::query!(
                    r#"
            INSERT INTO `Medals_Data` (
              `Medal_ID`, `Name`, `Link`, `Description`,
              `Gamemode`, `Grouping`, `Instructions`,
              `Ordering`
            )
            VALUES
              (?, ?, ?, ?, ?, ?, ?, ?) ON DUPLICATE KEY
            UPDATE
              `Medal_ID` = VALUES(`Medal_ID`),
              `Name` = VALUES(`Name`),
              `Link` = VALUES(`Link`),
              `Description` = VALUES(`Description`),
              `Gamemode` = VALUES(`Gamemode`),
              `Grouping` = VALUES(`Grouping`),
              `Instructions` = VALUES(`Instructions`),
              `Ordering` = VALUES(`Ordering`)"#,
                    id,
                    name.as_ref(),
                    link,
                    description.as_ref(),
                    mode.as_deref(),
                    grouping.as_ref(),
                    instructions.as_deref(),
                    ordering,
                );

                query
                    .execute(tx.deref_mut())
                    .await
                    .context("failed to execute Medals_Data query")?;
            }

            tx.commit()
                .await
                .context("failed to commit Medals_Data transaction")?;

            Ok(())
        }

        let res = inner(self, medals).await;
        let _entered = info_span!("store_medals").entered();

        match res {
            Ok(_) => info!("Successfully stored {} medals", medals.len()),
            Err(err) => error!(?err, "Failed to store medals"),
        }
    }

    #[must_use]
    pub fn store_rarities(&self, rarities: MedalRarities) -> JoinHandle<()> {
        async fn inner(db: Database, rarities: &MedalRarities) -> Result<()> {
            let mut tx = db
                .begin()
                .await
                .context("failed to begin transaction for Medals_Data")?;

            for (medal_id, MedalRarityEntry { count, frequency }) in rarities.iter() {
                let query = sqlx::query!(
                    r#"
UPDATE
  `Medals_Data`
SET
  `Frequency` = ?,
  `Count_Achieved_By` = ?
WHERE
  `Medal_ID` = ?"#,
                    frequency,
                    count,
                    medal_id,
                );

                query
                    .execute(tx.deref_mut())
                    .await
                    .context("failed to execute Medals_Data query")?;
            }

            tx.commit()
                .await
                .context("failed to commit Medals_Data transaction")?;

            Ok(())
        }

        let db = self.to_owned();

        tokio::spawn(async move {
            let res = inner(db, &rarities).await;
            let _entered = info_span!("store_rarities").entered();

            match res {
                Ok(_) => info!("Successfully stored {} medal rarities", rarities.len()),
                Err(err) => error!(?err, "Failed to store rarities"),
            }
        })
    }

    #[must_use]
    pub fn store_badges(&self, badges: Badges) -> JoinHandle<()> {
        async fn inner(db: Database, badges: &Badges) -> Result<()> {
            let mut tx = db
                .begin()
                .await
                .context("failed to begin transaction for badges")?;

            sqlx::query!("DELETE FROM `Badge_Name`")
                .execute(tx.deref_mut())
                .await
                .context("failed to delete rows in Badges_Users")?;

            for (BadgeDescription(description), entries) in badges.descriptions.iter() {
                for (BadgeName(name), owners) in entries.iter() {
                    for owner in owners {
                        let BadgeOwner {
                            user_id,
                            awarded_at,
                        } = owner;

                        let query = sqlx::query!(
                            r#"
                INSERT INTO `Badge_Name` (
                  `Name`, `User_ID`, `Description`, `Date_Awarded`
                )
                VALUES
                  (?, ?, ?, ?)"#,
                            name.as_ref(),
                            user_id,
                            description.as_ref(),
                            awarded_at,
                        );

                        query
                            .execute(tx.deref_mut())
                            .await
                            .context("failed to execute badge name query")?;
                    }
                }
            }

            for (BadgeName(name), BadgeImageUrl(image_url)) in badges.names.iter() {
                let query = sqlx::query!(
                    r#"
        INSERT INTO `Badges_Data` (
          `Name`, `Image_URL`
        )
        VALUES
          (?, ?)
        ON DUPLICATE KEY UPDATE
          `Name` = `Name`"#,
                    name.as_ref(),
                    image_url.as_ref(),
                );

                query
                    .execute(tx.deref_mut())
                    .await
                    .context("failed to execute badges data query")?;
            }

            tx.commit()
                .await
                .context("failed to commit Badges transaction")?;

            Ok(())
        }

        let db = self.to_owned();

        tokio::spawn(async move {
            let res = inner(db, &badges).await;
            let _entered = info_span!("store_badges").entered();

            match res {
                Ok(_) => info!("Successfully stored {} badges", badges.len()),
                Err(err) => error!(?err, "Failed to store badges"),
            }
        })
    }
}
