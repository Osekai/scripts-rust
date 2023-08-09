use std::ops::DerefMut;

use eyre::{Context as _, Result};
use sqlx::{Error as SqlxError, MySql, MySqlPool, Transaction};

use crate::model::{
    BadgeEntry, BadgeKey, Badges, MedalRarities, MedalRarityEntry, RankingUser, ScrapedMedal,
};

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

    async fn begin(&self) -> Result<Transaction<'_, MySql>, SqlxError> {
        self.mysql.begin().await
    }

    pub async fn store_rankings(&self, rankings: &[RankingUser]) -> Result<()> {
        let mut tx = self
            .begin()
            .await
            .context("failed to begin transaction for Ranking")?;

        for ranking in rankings {
            let RankingUser {
                id,
                name,
                stdev_acc,
                standard_acc,
                taiko_acc,
                ctb_acc,
                mania_acc,
                stdev_level,
                standard_level,
                taiko_level,
                ctb_level,
                mania_level,
                stdev_pp,
                standard_pp,
                taiko_pp,
                ctb_pp,
                mania_pp,
                medal_count,
                rarest_medal_id,
                rarest_medal_achieved,
                country_code,
                standard_global,
                taiko_global,
                ctb_global,
                mania_global,
                badge_count,
                ranked_maps,
                loved_maps,
                followers,
                subscribers,
                replays_watched,
                avatar_url,
                kudosu,
                restricted,
            } = ranking;

            let query = sqlx::query!(
                r#"
INSERT INTO Ranking (
  id, name, stdev_pp, standard_pp, taiko_pp, 
  ctb_pp, mania_pp, medal_count, rarest_medal, 
  country_code, standard_global, taiko_global, 
  ctb_global, mania_global, badge_count, 
  ranked_maps, loved_maps, subscribers, 
  followers, replays_watched, avatar_url, 
  rarest_medal_achieved, restricted, 
  stdev_acc, standard_acc, taiko_acc, 
  ctb_acc, mania_acc, stdev_level, 
  standard_level, taiko_level, ctb_level, 
  mania_level, kudosu
) 
VALUES 
  (
    ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 
    ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 
    ?, ?, ?
  ) ON DUPLICATE KEY 
UPDATE 
  id = VALUES(id), 
  name = VALUES(name), 
  stdev_pp = VALUES(stdev_pp), 
  standard_pp = VALUES(standard_pp), 
  taiko_pp = VALUES(taiko_pp), 
  ctb_pp = VALUES(ctb_pp), 
  mania_pp = VALUES(mania_pp), 
  medal_count = VALUES(medal_count), 
  rarest_medal = VALUES(rarest_medal), 
  country_code = VALUES(country_code), 
  standard_global = VALUES(standard_global), 
  taiko_global = VALUES(taiko_global), 
  ctb_global = VALUES(ctb_global), 
  mania_global = VALUES(mania_global), 
  badge_count = VALUES(badge_count), 
  ranked_maps = VALUES(ranked_maps), 
  loved_maps = VALUES(loved_maps), 
  subscribers = VALUES(subscribers), 
  followers = VALUES(followers), 
  replays_watched = VALUES(replays_watched), 
  avatar_url = VALUES(avatar_url), 
  rarest_medal_achieved = VALUES(rarest_medal_achieved), 
  restricted = VALUES(restricted), 
  stdev_acc = VALUES(stdev_acc), 
  standard_acc = VALUES(standard_acc), 
  taiko_acc = VALUES(taiko_acc), 
  ctb_acc = VALUES(ctb_acc), 
  mania_acc = VALUES(mania_acc), 
  stdev_level = VALUES(stdev_level), 
  standard_level = VALUES(standard_level), 
  taiko_level = VALUES(taiko_level), 
  ctb_level = VALUES(ctb_level), 
  mania_level = VALUES(mania_level), 
  kudosu = VALUES(kudosu)"#,
                id,
                name.as_ref(),
                stdev_pp,
                standard_pp,
                taiko_pp,
                ctb_pp,
                mania_pp,
                medal_count,
                rarest_medal_id,
                country_code.as_ref(),
                standard_global,
                taiko_global,
                ctb_global,
                mania_global,
                badge_count,
                ranked_maps,
                loved_maps,
                subscribers,
                followers,
                replays_watched,
                avatar_url.as_ref(),
                rarest_medal_achieved,
                restricted,
                stdev_acc,
                standard_acc,
                taiko_acc,
                ctb_acc,
                mania_acc,
                stdev_level,
                standard_level,
                taiko_level,
                ctb_level,
                mania_level,
                kudosu
            );

            query
                .execute(tx.deref_mut())
                .await
                .context("failed to execute Ranking query")?;
        }

        tx.commit()
            .await
            .context("failed to commit Ranking transaction")?;

        Ok(())
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
