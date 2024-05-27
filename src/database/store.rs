use std::{num::NonZeroU32, ops::DerefMut};

use eyre::{Context as _, Result};
use time::OffsetDateTime;
use tokio::task::JoinHandle;

use crate::model::{
    BadgeEntry, BadgeKey, Badges, Finish, MedalRarities, MedalRarityEntry, Progress, RankingUser,
    RankingsIter, ScrapedMedal,
};

use super::Database;

impl Database {
    pub async fn store_progress(&self, progress: &Progress) -> Result<()> {
        let mut conn = self
            .acquire()
            .await
            .context("failed to acquire connection to upsert Rankings_Script_History")?;

        let Progress {
            id,
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
            id,
            task.to_string(),
            start,
            *current as i32,
            *total as i32,
            eta_seconds,
        );

        query
            .execute(conn.deref_mut())
            .await
            .context("failed to execute RankingLoopInfo query")?;

        Ok(())
    }

    pub async fn store_finish(&self, finish: &Finish) -> Result<()> {
        // let mut tx = self
        //     .begin()
        //     .await
        //     .context("failed to begin transaction for RankingLoopHistory")?;

        // let Finish {
        //     requested_users,
        //     task,
        // } = finish;

        //         let insert_query = sqlx::query!(
        //             r#"
        // INSERT INTO RankingLoopHistory (Time, LoopType, Amount)
        // VALUES
        //   (CURRENT_TIMESTAMP, ?, ?)"#,
        //             task.to_string(),
        //             *requested_users as i32
        //         );

        //         insert_query
        //             .execute(tx.deref_mut())
        //             .await
        //             .context("failed to execute RankingLoopHistory query")?;

        //         let update_query = sqlx::query!(
        //             r#"
        // UPDATE
        //   RankingLoopInfo
        // SET
        //   CurrentLoop = "Complete"
        // LIMIT
        //   1"#
        //         );

        //         update_query
        //             .execute(tx.deref_mut())
        //             .await
        //             .context("failed to execute RankingLoopInfo query")?;

        //         tx.commit()
        //             .await
        //             .context("failed to commit finish transaction")?;

        Ok(())
    }

    #[must_use]
    pub fn store_rankings(&self, rankings: RankingsIter) -> JoinHandle<()> {
        async fn inner(db: Database, rankings: RankingsIter) -> Result<()> {
            //             let mut tx = db
            //                 .begin()
            //                 .await
            //                 .context("failed to begin transaction for Ranking")?;

            //             for ranking in rankings {
            //                 let stdev_acc = ranking.std_dev_acc();
            //                 let stdev_level = ranking.std_dev_level();
            //                 let stdev_pp = ranking.std_dev_pp();
            //                 let total_pp = ranking.total_pp();

            //                 let RankingUser {
            //                     id,
            //                     name,
            //                     ignore_acc,
            //                     medal_count,
            //                     rarest_medal_id,
            //                     rarest_medal_achieved,
            //                     country_code,
            //                     badge_count,
            //                     ranked_maps,
            //                     loved_maps,
            //                     followers,
            //                     subscribers,
            //                     replays_watched,
            //                     kudosu,
            //                     restricted,
            //                     std,
            //                     tko,
            //                     ctb,
            //                     mna,
            //                 } = ranking;

            //                 let mut std_acc = std.acc;
            //                 let mut tko_acc = tko.acc;
            //                 let mut ctb_acc = ctb.acc;
            //                 let mut mna_acc = mna.acc;

            //                 if ignore_acc {
            //                     std_acc = 0.0;
            //                     tko_acc = 0.0;
            //                     ctb_acc = 0.0;
            //                     mna_acc = 0.0;
            //                 }

            //                 let query = sqlx::query!(
            //                     r#"
            // INSERT INTO Ranking (
            //   id, name, total_pp, stdev_pp, standard_pp,
            //   taiko_pp, ctb_pp, mania_pp, medal_count,
            //   rarest_medal, country_code, standard_global,
            //   taiko_global, ctb_global, mania_global,
            //   badge_count, ranked_maps, loved_maps,
            //   subscribers, followers, replays_watched,
            //   rarest_medal_achieved, restricted,
            //   stdev_acc, standard_acc, taiko_acc,
            //   ctb_acc, mania_acc, stdev_level,
            //   standard_level, taiko_level, ctb_level,
            //   mania_level, kudosu, avatar_url
            // )
            // VALUES
            //   (
            //     ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?,
            //     ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?,
            //     ?, ?, ?, ?
            //   ) ON DUPLICATE KEY
            // UPDATE
            //   id = VALUES(id),
            //   name = VALUES(name),
            //   total_pp = VALUES(total_pp),
            //   stdev_pp = VALUES(stdev_pp),
            //   standard_pp = VALUES(standard_pp),
            //   taiko_pp = VALUES(taiko_pp),
            //   ctb_pp = VALUES(ctb_pp),
            //   mania_pp = VALUES(mania_pp),
            //   medal_count = VALUES(medal_count),
            //   rarest_medal = VALUES(rarest_medal),
            //   country_code = VALUES(country_code),
            //   standard_global = VALUES(standard_global),
            //   taiko_global = VALUES(taiko_global),
            //   ctb_global = VALUES(ctb_global),
            //   mania_global = VALUES(mania_global),
            //   badge_count = VALUES(badge_count),
            //   ranked_maps = VALUES(ranked_maps),
            //   loved_maps = VALUES(loved_maps),
            //   subscribers = VALUES(subscribers),
            //   followers = VALUES(followers),
            //   replays_watched = VALUES(replays_watched),
            //   rarest_medal_achieved = VALUES(rarest_medal_achieved),
            //   restricted = VALUES(restricted),
            //   stdev_acc = VALUES(stdev_acc),
            //   standard_acc = VALUES(standard_acc),
            //   taiko_acc = VALUES(taiko_acc),
            //   ctb_acc = VALUES(ctb_acc),
            //   mania_acc = VALUES(mania_acc),
            //   stdev_level = VALUES(stdev_level),
            //   standard_level = VALUES(standard_level),
            //   taiko_level = VALUES(taiko_level),
            //   ctb_level = VALUES(ctb_level),
            //   mania_level = VALUES(mania_level),
            //   kudosu = VALUES(kudosu)"#,
            //                     id,
            //                     name.as_ref(),
            //                     total_pp,
            //                     stdev_pp,
            //                     std.pp,
            //                     tko.pp,
            //                     ctb.pp,
            //                     mna.pp,
            //                     medal_count,
            //                     rarest_medal_id,
            //                     country_code.as_ref(),
            //                     std.global_rank.map(NonZeroU32::get),
            //                     tko.global_rank.map(NonZeroU32::get),
            //                     ctb.global_rank.map(NonZeroU32::get),
            //                     mna.global_rank.map(NonZeroU32::get),
            //                     badge_count,
            //                     ranked_maps,
            //                     loved_maps,
            //                     subscribers,
            //                     followers,
            //                     replays_watched,
            //                     rarest_medal_achieved,
            //                     restricted as u8,
            //                     stdev_acc,
            //                     std_acc,
            //                     tko_acc,
            //                     ctb_acc,
            //                     mna_acc,
            //                     stdev_level,
            //                     std.level,
            //                     tko.level,
            //                     ctb.level,
            //                     mna.level,
            //                     kudosu,
            //                     0_i32, // the avatar_url column is no longer needed
            //                 );

            //                 query
            //                     .execute(tx.deref_mut())
            //                     .await
            //                     .context("failed to execute Ranking query")?;
            //             }

            //             tx.commit()
            //                 .await
            //                 .context("failed to commit Ranking transaction")?;

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

    #[must_use]
    pub fn store_medals(&self, medals: Box<[ScrapedMedal]>) -> JoinHandle<()> {
        async fn inner(db: Database, medals: &[ScrapedMedal]) -> Result<()> {
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

                let query = sqlx::query!(
                    r#"
            INSERT INTO Medals_Data (
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
                    icon_url.as_ref(),
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

        let db = self.to_owned();

        tokio::spawn(async move {
            let res = inner(db, &medals).await;
            let _entered = info_span!("store_medals").entered();

            match res {
                Ok(_) => info!("Successfully stored {} medals", medals.len()),
                Err(err) => error!(?err, "Failed to store medals"),
            }
        })
    }

    #[must_use]
    pub fn store_rarities(&self, rarities: MedalRarities) -> JoinHandle<()> {
        async fn inner(db: Database, rarities: &MedalRarities) -> Result<()> {
            //             let mut tx = db
            //                 .begin()
            //                 .await
            //                 .context("failed to begin transaction for MedalRarity")?;

            //             for (medal_id, MedalRarityEntry { count, frequency }) in rarities.iter() {
            //                 let query = sqlx::query!(
            //                     r#"
            // INSERT INTO MedalRarity (id, frequency, count)
            // VALUES
            //   (?, ?, ?) ON DUPLICATE KEY
            // UPDATE
            //   id = VALUES(id),
            //   frequency = VALUES(frequency),
            //   count = VALUES(count)"#,
            //                     medal_id,
            //                     frequency,
            //                     count
            //                 );

            //                 query
            //                     .execute(tx.deref_mut())
            //                     .await
            //                     .context("failed to execute MedalRarity query")?;
            //             }

            //             tx.commit()
            //                 .await
            //                 .context("failed to commit MedalRarity transaction")?;

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
            //             let mut tx = db
            //                 .begin()
            //                 .await
            //                 .context("failed to begin transaction for Badges")?;

            //             sqlx::query!("DELETE FROM Badges")
            //                 .execute(tx.deref_mut())
            //                 .await
            //                 .context("failed to delete rows in Badges")?;

            //             for (key, value) in badges.iter() {
            //                 let BadgeKey { image_url } = key;

            //                 let BadgeEntry {
            //                     description,
            //                     id,
            //                     awarded_at,
            //                     users,
            //                 } = value;

            //                 let name = image_url
            //                     .rsplit_once('/')
            //                     .and_then(|(_, file)| file.rsplit_once('.'))
            //                     .map(|(name, _)| name.replace(['-', '_'], " "));

            //                 let query = sqlx::query!(
            //                     r#"
            // INSERT INTO Badges (
            //   id, name, image_url, description, awarded_at, users
            // )
            // VALUES
            //   (?, ?, ?, ?, ?, ?)"#,
            //                     id,
            //                     name,
            //                     image_url.as_ref(),
            //                     description.as_ref(),
            //                     awarded_at,
            //                     users.to_string(),
            //                 );

            //                 query
            //                     .execute(tx.deref_mut())
            //                     .await
            //                     .context("failed to execute Badges query")?;
            //             }

            //             tx.commit()
            //                 .await
            //                 .context("failed to commit Badges transaction")?;

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
