use std::{collections::HashMap, fs, string::FromUtf8Error};

use crate::model::{MedalRarity, ScrapedMedal, ScrapedUser, UserFull};

use super::Context;

use eyre::{Context as _, ContextCompat as _, Result};
use scraper::{Html, Selector};

impl Context {
    pub async fn gather_medals(&self) -> Result<Vec<ScrapedMedal>> {
        // Avoid request spamming while debugging
        let bytes = if cfg!(debug_assertion) {
            fs::read("./peppy.html").context("failed to read `./peppy.html`")?
        } else {
            let url = "https://osu.ppy.sh/users/peppy/osu";

            let bytes = self
                .client
                .send_get_request(url)
                .await
                .context("failed to request user to gather medals")?;

            // fs::write("./peppy.html", &bytes).unwrap();

            bytes.into()
        };

        let html_str = String::from_utf8(bytes)
            .map_err(FromUtf8Error::into_bytes)
            .map_err(|bytes| eyre!("received invalid UTF-8 while gathering medals: {bytes:?}"))?;

        let html = Html::parse_fragment(&html_str);
        let divs = Selector::parse("div").expect("invalid selector");

        let data = html
            .select(&divs)
            .find_map(|div| div.value().attr("data-initial-data"))
            .context("missing div with attribute `data-initial-data`")?;

        let deserialized: ScrapedUser = serde_json::from_str(&data)
            .with_context(|| format!("failed to deserialize scraped data: {data}"))?;

        Ok(deserialized.medals)
    }

    pub fn calculate_medal_rarity(users: &[UserFull], medals: &[ScrapedMedal]) -> Vec<MedalRarity> {
        let mut counts = users.iter().filter_map(UserFull::medals).flatten().fold(
            HashMap::<_, u32>::with_capacity(200),
            |mut counts, medal| {
                *counts.entry(medal.medal_id).or_default() += 1;

                counts
            },
        );

        for medal in medals {
            counts.entry(medal.id).or_insert(0);
        }

        let user_count = users.len() as f64;

        counts
            .into_iter()
            .map(|(medal_id, count)| MedalRarity {
                medal_id,
                frequency: (100 * count) as f64 / user_count,
            })
            .collect()
    }
}
