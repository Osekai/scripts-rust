use rosu_v2::prelude::{Badge, MedalCompact, User};

use super::MedalRarities;

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

    #[cfg(feature = "generate")]
    pub fn medals_mut(&mut self) -> Option<&mut Vec<MedalCompact>> {
        self.inner[0].medals.as_mut()
    }

    pub fn rarest_medal_id(&self, rarities: &MedalRarities) -> Option<u32> {
        self.medals()?
            .iter()
            .flat_map(|medal| Some((medal.medal_id, rarities.get(&medal.medal_id)?.count)))
            .max_by_key(|(_, count)| *count)
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
