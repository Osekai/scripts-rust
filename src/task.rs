use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    ops::{BitAndAssign, BitOr, BitOrAssign, Not},
    str::FromStr,
};

use eyre::Report;
use serde::{Serialize, Serializer};

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Task(u8);

#[rustfmt::skip]
impl Task {
    pub const MEDALS: Self =  Self(1 << 0);
    pub const BADGES: Self =  Self(1 << 1);
    pub const RARITY: Self =  Self(1 << 2);
    pub const RANKING: Self = Self(1 << 3);

    pub const DEFAULT: Self =
        Self(Self::MEDALS.0 | Self::BADGES.0 | Self::RANKING.0);
    pub const FULL: Self = Self(Self::DEFAULT.0 | Self::RARITY.0);
}

impl Task {
    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub fn remove(&mut self, other: Self) {
        *self &= !other;
    }

    pub fn empty() -> Self {
        Self(0)
    }

    /// Should all medals be retrieved and uploaded?
    pub fn medals(self) -> bool {
        self.contains(Self::MEDALS)
    }

    /// Should badges be processed and uploaded?
    pub fn badges(self) -> bool {
        self.contains(Self::BADGES)
    }

    /// Should the leaderboards be requested so that
    /// medal rarities are calculated and uploaded?
    pub fn rarity(self) -> bool {
        self.contains(Self::RARITY)
    }

    /// Should user data be processed and uploaded?
    pub fn ranking(self) -> bool {
        self.contains(Self::RANKING)
    }
}

impl Display for Task {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let mut found = false;
        let mut task = *self;

        if task.contains(Self::FULL) {
            return f.write_str("Full");
        }

        if task.contains(Self::DEFAULT) {
            f.write_str("Default")?;
            found = true;
            task.remove(Self::DEFAULT);
        }

        if task.contains(Self::MEDALS) {
            if found {
                f.write_str(" | ")?;
            }

            f.write_str("Medals")?;
            found = true;
            task.remove(Self::MEDALS);
        }

        if task.contains(Self::BADGES) {
            if found {
                f.write_str(" | ")?;
            }

            f.write_str("Badges")?;
            found = true;
            task.remove(Self::BADGES);
        }

        if task.contains(Self::RARITY) {
            if found {
                f.write_str(" | ")?;
            }

            f.write_str("Rarity")?;
            found = true;
            task.remove(Self::RARITY);
        }

        if task.contains(Self::RANKING) {
            if found {
                f.write_str(" | ")?;
            }

            f.write_str("Ranking")?;
        }

        Ok(())
    }
}

impl FromStr for Task {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_ascii_lowercase();

        s.split('|')
            .map(str::trim)
            .try_fold(Self::empty(), |res, next| match next {
                "default" => Ok(res | Self::DEFAULT),
                "full" => Ok(res | Self::FULL),
                "medal" | "medals" => Ok(res | Self::MEDALS),
                "rarity" | "rarities" => Ok(res | Self::RARITY),
                "ranking" => Ok(res | Self::RANKING),
                "badge" | "badges" => Ok(res | Self::BADGES),
                _ => {
                    let msg = format!(
                        "failed to parse task `{s}`; must be a `|`-separated list of the following: \
                        default, full, medal, rarity, badge, ranking"
                    );

                    Err(Report::msg(msg))
                }
            })
    }
}

impl BitOr for Task {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for Task {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAndAssign for Task {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl Not for Task {
    type Output = Self;

    #[inline]
    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl Serialize for Task {
    #[inline]
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}
