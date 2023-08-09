use std::{
    collections::{hash_map::Iter, HashMap},
    fmt::{Formatter, Result as FmtResult},
    iter::FromIterator,
};

use serde::{
    de::{DeserializeSeed, Error as SerdeError, MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer,
};

use crate::util::IntHasher;

pub struct MedalRarityEntry {
    pub count: u32,
    pub frequency: f32,
}

#[derive(Default)]
pub struct MedalRarities {
    inner: HashMap<u16, MedalRarityEntry, IntHasher>,
}

impl MedalRarities {
    pub fn get(&self, medal_id: &u16) -> Option<&MedalRarityEntry> {
        self.inner.get(medal_id)
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn iter(&self) -> Iter<'_, u16, MedalRarityEntry> {
        self.inner.iter()
    }
}

impl FromIterator<(u16, u32, f32)> for MedalRarities {
    #[inline]
    fn from_iter<T: IntoIterator<Item = (u16, u32, f32)>>(iter: T) -> Self {
        let inner = iter
            .into_iter()
            .map(|(medal_id, count, frequency)| {
                let entry = MedalRarityEntry { count, frequency };

                (medal_id, entry)
            })
            .collect();

        Self { inner }
    }
}

impl<'de> Deserialize<'de> for MedalRarities {
    #[inline]
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_seq(MedalRaritiesVisitor)
    }
}

struct MedalRaritiesVisitor;

impl<'de> Visitor<'de> for MedalRaritiesVisitor {
    type Value = MedalRarities;

    fn expecting(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str("a list of medal rarities")
    }

    #[inline]
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let capacity = seq.size_hint().unwrap_or(0);
        let mut inner = HashMap::with_capacity_and_hasher(capacity, IntHasher);

        while seq.next_element_seed(RaritySeed(&mut inner))?.is_some() {}

        Ok(MedalRarities { inner })
    }
}

struct RaritySeed<'m>(&'m mut HashMap<u16, MedalRarityEntry, IntHasher>);

impl<'de> DeserializeSeed<'de> for RaritySeed<'_> {
    type Value = ();

    #[inline]
    fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<Self::Value, D::Error> {
        d.deserialize_map(self)
    }
}

impl<'de> Visitor<'de> for RaritySeed<'_> {
    type Value = ();

    fn expecting(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str("a map containing `id`, `frequency`, and `count` fields")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut medal_id = None;
        let mut frequency = None;
        let mut count = None;

        while let Some(key) = map.next_key()? {
            match key {
                "id" => medal_id = Some(map.next_value()?),
                "frequency" => frequency = Some(map.next_value()?),
                "count" => count = Some(map.next_value()?),
                _ => {
                    return Err(SerdeError::unknown_field(
                        key,
                        &["id", "frequency", "count"],
                    ))
                }
            }
        }

        let medal_id = medal_id.ok_or_else(|| SerdeError::missing_field("medalid"))?;

        let entry = MedalRarityEntry {
            count: count.ok_or_else(|| SerdeError::missing_field("count"))?,
            frequency: frequency.ok_or_else(|| SerdeError::missing_field("frequency"))?,
        };

        self.0.insert(medal_id, entry);

        Ok(())
    }
}
