use std::collections::{hash_map::Iter, HashMap};

use crate::{model::ScrapedMedal, util::IntHasher};

#[derive(Copy, Clone, Debug)]
pub struct MedalRarityEntry {
    pub count: u32,
    pub frequency: f64,
}

#[derive(Clone, Default)]
pub struct MedalRarities {
    inner: HashMap<u16, MedalRarityEntry, IntHasher>,
}

impl MedalRarities {
    pub fn extract(medals: &[ScrapedMedal]) -> Self {
        let mut inner = HashMap::with_capacity_and_hasher(500, IntHasher);

        let iter = medals.iter().map(|medal| {
            (
                medal.id,
                MedalRarityEntry {
                    count: medal.achieved_count,
                    frequency: medal.achieved_percent,
                },
            )
        });

        inner.extend(iter);

        Self { inner }
    }

    pub fn get(&self, medal_id: &u16) -> Option<&MedalRarityEntry> {
        self.inner.get(medal_id)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn iter(&self) -> Iter<'_, u16, MedalRarityEntry> {
        self.inner.iter()
    }
}
