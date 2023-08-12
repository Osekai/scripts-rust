use std::{
    collections::{hash_map::Iter, HashMap},
    iter::FromIterator,
};

use crate::util::IntHasher;

#[derive(Copy, Clone)]
pub struct MedalRarityEntry {
    pub count: u32,
    pub frequency: f32,
}

#[derive(Clone, Default)]
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

impl Extend<(u16, u32, f32)> for MedalRarities {
    fn extend<T: IntoIterator<Item = (u16, u32, f32)>>(&mut self, iter: T) {
        let iter = iter
            .into_iter()
            .map(|(medal_id, count, frequency)| (medal_id, MedalRarityEntry { count, frequency }));

        self.inner.extend(iter)
    }
}
