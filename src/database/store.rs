use std::{collections::HashMap, num::NonZeroU32, ops::DerefMut};

use eyre::{Context as _, Result};
use sqlx::QueryBuilder;
use tokio::task::JoinHandle;

use crate::model::{
    BadgeDescription, BadgeImageUrl, BadgeName, BadgeOwner, Badges, Finish, MedalRarities,
    MedalRarityEntry, OsuUser, Progress, RankingUser, RankingsIter, ScrapedMedal,
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

    pub async fn update_usernames(&self, users: &[OsuUser]) {
        async fn inner(db: &Database, users: &[OsuUser]) -> Result<()> {
            let mut tx = db
                .begin()
                .await
                .context("failed to begin transaction for System_Users update")?;

            for user in users {
                let OsuUser::Available(ref user) = user else {
                    continue;
                };

                let query = sqlx::query!(
                    r#"UPDATE `System_Users` SET `Name` = ? WHERE `User_ID` = ?"#,
                    user.username,
                    user.user_id,
                );

                query
                    .execute(tx.deref_mut())
                    .await
                    .context("failed to execute System_Users update")?;
            }

            tx.commit()
                .await
                .context("failed to commit System_Users transaction")?;

            Ok(())
        }

        let res = inner(self, users).await;
        let _entered = info_span!("update_usernames").entered();

        match res {
            Ok(_) => info!("Successfully updated usernames"),
            Err(err) => error!(?err, "Failed to update usernames"),
        }
    }

    // This method is async instead of returning a JoinHandle because the
    // caller can only provide users by reference at this point.
    pub async fn store_user_medals(&self, users: &[OsuUser]) {
        async fn inner(db: &Database, users: &[OsuUser]) -> Result<usize> {
            let mut len = 0;

            let mut tx = db
                .begin()
                .await
                .context("failed to begin transaction for Rankings_Users_Medals")?;

            for user in users {
                let OsuUser::Available(ref user) = user else {
                    continue;
                };

                if user.medals.is_empty() {
                    continue;
                }

                let mut qb = QueryBuilder::new(
                    "REPLACE INTO `Rankings_Users_Medals` (`User_ID`, `Medal_ID`, `Achieved_At`) ",
                );

                let query = qb
                    .push_values(user.medals.iter(), |mut b, medal| {
                        b.push_bind(user.user_id)
                            .push_bind(medal.medal_id)
                            .push_bind(medal.achieved_at);
                    })
                    .build();

                query.execute(tx.deref_mut()).await.with_context(|| {
                    format!(
                        "failed to execute Rankings_Users_Medals query user_id={}",
                        user.user_id
                    )
                })?;

                len += user.medals.len();
            }

            tx.commit()
                .await
                .context("failed to commit Rankings_Users_Medals transaction")?;

            Ok(len)
        }

        let res = inner(self, users).await;
        let _entered = info_span!("store_user_medals").entered();

        match res {
            Ok(len) => info!("Successfully stored {len} user medals"),
            Err(err) => error!(?err, "Failed to store user medals"),
        }
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
                    kudosu_total,
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
                `Rank_Global_Taiko`, `Rarest_Medal_Achieved`, `Rarest_Medal_ID`,
                `Count_SS_Catch`, `Count_SS_Mania`, `Count_SS_Standard`, `Count_SS_Taiko`,
                `Count_SSH_Catch`, `Count_SSH_Mania`, `Count_SSH_Standard`, `Count_SSH_Taiko`,
                `Count_S_Catch`, `Count_S_Mania`, `Count_S_Standard`, `Count_S_Taiko`,
                `Count_SH_Catch`, `Count_SH_Mania`, `Count_SH_Standard`, `Count_SH_Taiko`,
                `Count_A_Catch`, `Count_A_Mania`, `Count_A_Standard`, `Count_A_Taiko`,
                `Total_Hits_Catch`, `Total_Hits_Mania`, `Total_Hits_Standard`, `Total_Hits_Taiko`,
                `Play_Time_Catch`, `Play_Time_Mania`, `Play_Time_Standard`, `Play_Time_Taiko`,
                `Play_Count_Catch`, `Play_Count_Mania`, `Play_Count_Standard`, `Play_Count_Taiko`,
                `Total_Score_Catch`, `Total_Score_Mania`, `Total_Score_Standard`, `Total_Score_Taiko`,
                `Kudosu_Total`
            )
            VALUES
              (
                ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?,
                ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?,
                ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?,
                ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?,
                ?, ?, ?
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
                `Rarest_Medal_ID` = VALUES(`Rarest_Medal_ID`),
                `Count_SS_Catch` = VALUES(`Count_SS_Catch`),
                `Count_SS_Mania` = VALUES(`Count_SS_Mania`),
                `Count_SS_Standard` = VALUES(`Count_SS_Standard`),
                `Count_SS_Taiko` = VALUES(`Count_SS_Taiko`),
                `Count_SSH_Catch` = VALUES(`Count_SSH_Catch`),
                `Count_SSH_Mania` = VALUES(`Count_SSH_Mania`),
                `Count_SSH_Standard` = VALUES(`Count_SSH_Standard`),
                `Count_SSH_Taiko` = VALUES(`Count_SSH_Taiko`),
                `Count_S_Catch` = VALUES(`Count_S_Catch`),
                `Count_S_Mania` = VALUES(`Count_S_Mania`),
                `Count_S_Standard` = VALUES(`Count_S_Standard`),
                `Count_S_Taiko` = VALUES(`Count_S_Taiko`),
                `Count_SH_Catch` = VALUES(`Count_SH_Catch`),
                `Count_SH_Mania` = VALUES(`Count_SH_Mania`),
                `Count_SH_Standard` = VALUES(`Count_SH_Standard`),
                `Count_SH_Taiko` = VALUES(`Count_SH_Taiko`),
                `Count_A_Catch` = VALUES(`Count_A_Catch`),
                `Count_A_Mania` = VALUES(`Count_A_Mania`),
                `Count_A_Standard` = VALUES(`Count_A_Standard`),
                `Count_A_Taiko` = VALUES(`Count_A_Taiko`),
                `Total_Hits_Catch` = VALUES(`Total_Hits_Catch`),
                `Total_Hits_Mania` = VALUES(`Total_Hits_Mania`),
                `Total_Hits_Standard` = VALUES(`Total_Hits_Standard`),
                `Total_Hits_Taiko` = VALUES(`Total_Hits_Taiko`),
                `Play_Time_Catch` = VALUES(`Play_Time_Catch`),
                `Play_Time_Mania` = VALUES(`Play_Time_Mania`),
                `Play_Time_Standard` = VALUES(`Play_Time_Standard`),
                `Play_Time_Taiko` = VALUES(`Play_Time_Taiko`),
                `Play_Count_Catch` = VALUES(`Play_Count_Catch`),
                `Play_Count_Mania` = VALUES(`Play_Count_Mania`),
                `Play_Count_Standard` = VALUES(`Play_Count_Standard`),
                `Play_Count_Taiko` = VALUES(`Play_Count_Taiko`),
                `Total_Score_Catch` = VALUES(`Total_Score_Catch`),
                `Total_Score_Mania` = VALUES(`Total_Score_Mania`),
                `Total_Score_Standard` = VALUES(`Total_Score_Standard`),
                `Total_Score_Taiko` = VALUES(`Total_Score_Taiko`),
                `Kudosu_Total` = VALUES(`Kudosu_Total`)"#,
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
                    ctb.count_ss,
                    mna.count_ss,
                    std.count_ss,
                    tko.count_ss,
                    ctb.count_ssh,
                    mna.count_ssh,
                    std.count_ssh,
                    tko.count_ssh,
                    ctb.count_s,
                    mna.count_s,
                    std.count_s,
                    tko.count_s,
                    ctb.count_sh,
                    mna.count_sh,
                    std.count_sh,
                    tko.count_sh,
                    ctb.count_a,
                    mna.count_a,
                    std.count_a,
                    tko.count_a,
                    ctb.total_hits,
                    mna.total_hits,
                    std.total_hits,
                    tko.total_hits,
                    ctb.playtime,
                    mna.playtime,
                    std.playtime,
                    tko.playtime,
                    ctb.playcount,
                    mna.playcount,
                    std.playcount,
                    tko.playcount,
                    ctb.total_score,
                    mna.total_score,
                    std.total_score,
                    tko.total_score,
                    kudosu_total,
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
                    achieved_count: _,
                    achieved_percent: _,
                    icon_url,
                    id,
                    name,
                    grouping,
                    ordering,
                    description,
                    mode,
                    instructions,
                } = medal;

                let link = icon_url.rsplit('/').next().unwrap_or(icon_url);

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

            sqlx::query!("DELETE FROM `Badges_Users`")
                .execute(tx.deref_mut())
                .await
                .context("failed to delete rows in Badges_Users")?;

            // Maps badge names to badge ids
            let mut indices = HashMap::new();
            // The badge id for the next new badge
            let mut index = 0;

            let mut badge_id = |name: &str| match indices.get(name) {
                Some(id) => *id,
                None => {
                    let id = index;
                    index += 1;
                    indices.insert(name.to_owned(), id);

                    id
                }
            };

            for (BadgeDescription(description), entries) in badges.descriptions.iter() {
                for (BadgeName(name), owners) in entries.iter() {
                    for owner in owners {
                        let BadgeOwner {
                            user_id,
                            awarded_at,
                        } = owner;

                        let badge_id = badge_id(name);

                        let query = sqlx::query!(
                            "
        INSERT INTO `Badges_Users` (`Badge_ID`, `User_ID`, `Description`, `Date_Awarded`)
        VALUES (?, ?, ?, ?)",
                            badge_id,
                            user_id,
                            description.as_ref(),
                            awarded_at,
                        );

                        query
                            .execute(tx.deref_mut())
                            .await
                            .context("failed to execute badges users query")?;
                    }
                }
            }

            sqlx::query!("DELETE FROM `Badges_Data`")
                .execute(tx.deref_mut())
                .await
                .context("failed to delete rows in Badges_Data")?;

            for (BadgeName(name), BadgeImageUrl(image_url)) in badges.names.iter() {
                let id = badge_id(name);

                let query = sqlx::query!(
                    "INSERT INTO `Badges_Data` (`ID`, `Name`, `Image_URL`) VALUES (?, ?, ?)",
                    id,
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
