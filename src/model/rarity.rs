use std::{collections::HashMap, iter::FromIterator};

use serde::{
    ser::{SerializeMap, SerializeSeq},
    Serialize, Serializer,
};

use crate::util::IntHasher;

pub struct MedalRarityEntry {
    pub count: usize,
    pub frequency: f64,
}

#[derive(Default)]
pub struct MedalRarities {
    inner: HashMap<u32, MedalRarityEntry, IntHasher>,
}

impl MedalRarities {
    pub fn get(&self, medal_id: &u32) -> Option<&MedalRarityEntry> {
        self.inner.get(medal_id)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

impl FromIterator<(u32, usize, f64)> for MedalRarities {
    #[inline]
    fn from_iter<T: IntoIterator<Item = (u32, usize, f64)>>(iter: T) -> Self {
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

impl Serialize for MedalRarities {
    #[inline]
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut s = s.serialize_seq(Some(self.inner.len()))?;

        for (key, value) in self.inner.iter() {
            let entry = BorrowedRarity::new(*key, value);
            s.serialize_element(&entry)?;
        }

        s.end()
    }
}

struct BorrowedRarity<'r> {
    medal_id: u32,
    rarity: &'r MedalRarityEntry,
}

impl<'r> BorrowedRarity<'r> {
    fn new(medal_id: u32, rarity: &'r MedalRarityEntry) -> Self {
        Self { medal_id, rarity }
    }
}

impl Serialize for BorrowedRarity<'_> {
    #[inline]
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut s = s.serialize_map(Some(3))?;

        s.serialize_entry("medalid", &self.medal_id)?;
        s.serialize_entry("count", &self.rarity.count)?;
        s.serialize_entry("frequency", &self.rarity.frequency)?;

        s.end()
    }
}
