use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    ops::{BitAndAssign, BitOr, BitOrAssign, Not},
    str::FromStr,
};

use eyre::Report;

#[derive(Copy, Clone, Debug)]
pub struct Task(u8);

#[rustfmt::skip]
impl Task {
    pub const MEDALS: Self =       Self(1 << 0);
    pub const LEADERBOARD: Self =  Self(1 << 1);
    pub const BADGES: Self =       Self(1 << 2);
    pub const RARITY: Self =       Self(1 << 3);
    pub const RANKING: Self =      Self(1 << 4);
    pub const EXTRA_BADGES: Self = Self(1 << 5);

    pub const DEFAULT: Self =
        Self(Self::MEDALS.0 | Self::BADGES.0 | Self::RARITY.0 | Self::RANKING.0);
    pub const FULL: Self = Self(u8::MAX);
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

    pub fn medals(self) -> bool {
        self.contains(Self::MEDALS)
    }

    pub fn leaderboard(self) -> bool {
        self.contains(Self::LEADERBOARD)
    }

    pub fn badges(self) -> bool {
        self.contains(Self::BADGES)
    }

    pub fn rarity(self) -> bool {
        self.contains(Self::RARITY)
    }

    pub fn ranking(self) -> bool {
        self.contains(Self::RANKING)
    }

    pub fn extra_badges(self) -> bool {
        self.contains(Self::EXTRA_BADGES)
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

        if task.contains(Self::LEADERBOARD) {
            if found {
                f.write_str(" | ")?;
            }

            f.write_str("Leaderboard")?;
            found = true;
            task.remove(Self::LEADERBOARD);
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

        if task.contains(Self::EXTRA_BADGES) {
            if found {
                f.write_str(" | ")?;
            }

            f.write_str("ExtraBadges")?;
            found = true;
            task.remove(Self::EXTRA_BADGES);
        }

        Ok(())
    }
}

impl FromStr for Task {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_ascii_lowercase();

        let mut res = Self::empty();

        if s.contains("default") {
            res |= Self::DEFAULT;
        }

        if s.contains("full") {
            res |= Self::FULL;
        }

        if s.contains("medal") {
            res |= Self::MEDALS;
        }

        if s.contains("leaderboard") || s.contains("lb") {
            res |= Self::LEADERBOARD;
        }

        if s.contains("rarity") | s.contains("rarities") {
            res |= Self::RARITY;
        }

        // Note: will also hit for "extra badges"
        if s.contains("badge") {
            res |= Self::BADGES;
        }

        if s.contains("extra") {
            res |= Self::EXTRA_BADGES;
        }

        if res.0 == 0 {
            let msg = format!(
                "Failed to parse task `{s}`; must contain either of the following: \n\
                default, full, medal, leaderboard, rarity, badge, extra"
            );

            Err(Report::msg(msg))
        } else {
            Ok(res)
        }
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
