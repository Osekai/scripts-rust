use std::{
    collections::{HashMap, HashSet},
    fmt::{Formatter, Result as FmtResult},
    string::FromUtf8Error,
};

use eyre::{Context as _, ContextCompat as _, Result};
use scraper::{Html, Selector};
use serde::{
    de::{DeserializeSeed, Error as SerdeError, IgnoredAny, MapAccess, SeqAccess, Visitor},
    Deserializer as DeserializerTrait,
};
use serde_json::{de::SliceRead, Deserializer};

use crate::{
    model::{MedalRarities, OsuUser, ScrapedMedal, ScrapedUser},
    util::IntHasher,
};

use super::Context;

impl Context {
    pub async fn request_medals(&self) -> Result<Box<[ScrapedMedal]>> {
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

    pub async fn request_osekai_medals(&self) -> Result<HashSet<u16, IntHasher>> {
        let bytes = self
            .client
            .get_osekai_medals()
            .await
            .context("failed to get osekai medals")?;

        Deserializer::new(SliceRead::new(&bytes))
            .deserialize_seq(MedalsVisitor)
            .with_context(|| {
                let text = String::from_utf8_lossy(&bytes);

                format!("failed to deserialize osekai medals: {text}")
            })
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
    pub fn calculate_rarities(users: &[OsuUser], medals: &[ScrapedMedal]) -> MedalRarities {
        let mut counts = HashMap::with_capacity_and_hasher(200, IntHasher);

        let user_medals = users
            .iter()
            .filter_map(|user| match user {
                OsuUser::Available(user) => Some(user),
                OsuUser::Restricted { .. } => None,
            })
            .flat_map(|user| user.medals.iter());

        for medal in user_medals {
            *counts.entry(medal.medal_id as u16).or_default() += 1;
        }

        // In case no user owns the medal yet, still add it as an entry
        for medal in medals {
            counts.entry(medal.id).or_insert(0);
        }

        let user_count = users.len() as f32;

        counts
            .into_iter()
            .map(|(medal_id, count)| (medal_id, count, (100 * count) as f32 / user_count))
            .collect()
    }
}

struct MedalsVisitor;

impl<'de> Visitor<'de> for MedalsVisitor {
    type Value = HashSet<u16, IntHasher>;

    fn expecting(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str("a list containing objects with a medalid field")
    }

    #[inline]
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut medals = HashSet::with_capacity_and_hasher(300, IntHasher);

        while seq.next_element_seed(MedalId(&mut medals))?.is_some() {}

        Ok(medals)
    }
}

struct MedalId<'m>(&'m mut HashSet<u16, IntHasher>);

impl<'de> DeserializeSeed<'de> for MedalId<'_> {
    type Value = ();

    #[inline]
    fn deserialize<D: DeserializerTrait<'de>>(self, d: D) -> Result<Self::Value, D::Error> {
        d.deserialize_map(self)
    }
}

impl<'de> Visitor<'de> for MedalId<'_> {
    type Value = ();

    fn expecting(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str("an object with a medalid field")
    }

    #[inline]
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut medal_id = None;

        while let Some(key) = map.next_key::<&str>()? {
            if key == "medalid" {
                medal_id = Some(map.next_value()?);
            } else {
                let _: IgnoredAny = map.next_value()?;
            }
        }

        let medal_id = medal_id.ok_or_else(|| SerdeError::missing_field("medalid"))?;
        self.0.insert(medal_id);

        Ok(())
    }
}
