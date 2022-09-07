use std::{cmp::Ordering, collections::HashMap};

use rosu_v2::prelude::{Badge, MedalCompact, User};

use super::IntHasher;

pub struct UserFull {
    pub(super) inner: [User; 4],
}

impl UserFull {
    pub fn badges_mut(&mut self) -> Option<&mut [Badge]> {
        self.inner[0].badges.as_deref_mut()
    }

    pub fn medals(&self) -> Option<&[MedalCompact]> {
        self.inner[0].medals.as_deref()
    }

    pub fn rarest_medal_id(&self, rarities: &HashMap<u32, f64, IntHasher>) -> Option<u32> {
        self.medals()?
            .iter()
            .flat_map(|medal| Some((medal.medal_id, *rarities.get(&medal.medal_id)?)))
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(Ordering::Equal))
            .map(|(id, _)| id)
    }

    pub fn pp_iter(&self) -> impl Iterator<Item = f32> + '_ {
        self.inner
            .iter()
            .filter_map(|user| user.statistics.as_ref())
            .map(|stats| stats.pp)
    }

    pub fn total_pp(&self) -> f32 {
        self.pp_iter().sum()
    }

    pub fn std_dev_pp(&self) -> f32 {
        let total = self.total_pp();
        let mean = total / 4.0;
        let variance: f32 = self.pp_iter().map(|pp| (pp - mean) * (pp - mean)).sum();
        let std_dev = (variance / 3.0).sqrt();

        total - 2.0 * std_dev
    }
}

impl From<[User; 4]> for UserFull {
    #[inline]
    fn from(inner: [User; 4]) -> Self {
        Self { inner }
    }
}
