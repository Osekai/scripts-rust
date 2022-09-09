use std::{collections::HashMap, string::FromUtf8Error};

use crate::{
    model::{MedalRarities, ScrapedMedal, ScrapedUser, UserFull},
    util::IntHasher,
};

use super::Context;

use eyre::{Context as _, ContextCompat as _, Result};
use scraper::{Html, Selector};

impl Context {
    pub async fn request_medals(&self) -> Result<Vec<ScrapedMedal>> {
        let bytes = self
            .client
            .get_user_webpage()
            .await
            .context("failed to get user to gather medals")?;

        let html_str = String::from_utf8(bytes)
            .map_err(FromUtf8Error::into_bytes)
            .map_err(|bytes| eyre!("received invalid UTF-8 while gathering medals: {bytes:?}"))?;

        let html = Html::parse_fragment(&html_str);
        let divs = Selector::parse("div").expect("invalid selector");

        let data = html
            .select(&divs)
            .find_map(|div| div.value().attr("data-initial-data"))
            .context("missing div with attribute `data-initial-data`")?;

        let deserialized: ScrapedUser = serde_json::from_str(data)
            .with_context(|| format!("failed to deserialize scraped data: {data}"))?;

        Ok(deserialized.medals)
    }

    /// Request all medal rarities stored by osekai
    pub async fn request_osekai_rarities(&self) -> Result<MedalRarities> {
        let bytes = self
            .client
            .get_osekai_rarities()
            .await
            .context("failed to get osekai rarities")?;

        serde_json::from_slice(&bytes).with_context(|| {
            let text = String::from_utf8_lossy(&bytes);

            format!("failed to deserialize osekai rarities: {text}")
        })
    }

    /// Calculate each medal's rarity i.e. how many users obtained it
    pub fn calculate_rarities(users: &[UserFull], medals: &[ScrapedMedal]) -> MedalRarities {
        let mut counts = HashMap::with_capacity_and_hasher(200, IntHasher);

        for medal in users.iter().filter_map(UserFull::medals).flatten() {
            *counts.entry(medal.medal_id).or_default() += 1;
        }

        // In case no user owns the medal yet, still add it as an entry
        for medal in medals {
            counts.entry(medal.id).or_insert(0);
        }

        let user_count = users.len() as f64;

        counts
            .into_iter()
            .map(|(medal_id, count)| (medal_id, count, (100 * count) as f64 / user_count))
            .collect()
    }
}
