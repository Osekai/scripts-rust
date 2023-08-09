use std::{
    cmp::Ordering,
    collections::{
        hash_map::{Iter, IterMut},
        HashMap, HashSet,
    },
    fmt::{Display, Formatter, Result as FmtResult},
    mem,
};

use rosu_v2::prelude::Badge;
use serde::{
    de::{Deserializer, Error as SerdeError, Unexpected},
    Deserialize,
};
use time::OffsetDateTime;

use crate::util::IntHasher;

// Different badges may have the same description so we
// use the image url as key instead.
//
// See github issue #1
#[derive(Eq, PartialEq, Hash)]
pub struct BadgeKey {
    pub image_url: Box<str>,
}

pub struct BadgeEntry {
    pub description: Box<str>,
    pub id: Option<u32>,
    pub awarded_at: OffsetDateTime,
    pub users: BadgeOwners,
}

#[derive(Default)]
pub struct Badges {
    inner: HashMap<BadgeKey, BadgeEntry>,
}

impl Badges {
    pub fn with_capacity(capacity: usize) -> Self {
        let inner = HashMap::with_capacity(capacity);

        Self { inner }
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn insert(&mut self, user_id: u32, badge: &mut Badge) {
        let key = BadgeKey {
            image_url: mem::take(&mut badge.image_url).into_boxed_str(),
        };

        let entry = self.inner.entry(key).or_insert_with(|| BadgeEntry {
            description: mem::take(&mut badge.description).into_boxed_str(),
            awarded_at: badge.awarded_at,
            users: BadgeOwners::default(),
            id: None,
        });

        entry.users.insert(user_id);
    }

    pub fn iter(&self) -> Iter<'_, BadgeKey, BadgeEntry> {
        self.inner.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, BadgeKey, BadgeEntry> {
        self.inner.iter_mut()
    }
}

pub struct BadgeOwners(HashSet<u32, IntHasher>);

impl BadgeOwners {
    fn insert(&mut self, user_id: u32) {
        self.0.insert(user_id);
    }

    pub fn extend(&mut self, user_ids: &[u32]) {
        self.0.extend(user_ids);
    }
}

impl Default for BadgeOwners {
    fn default() -> Self {
        Self(HashSet::with_hasher(IntHasher))
    }
}

impl Display for BadgeOwners {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str("[")?;

        let mut iter = self.0.iter();

        if let Some(elem) = iter.next() {
            Display::fmt(elem, f)?;

            for elem in iter {
                write!(f, ",{elem}")?;
            }
        }

        f.write_str("]")
    }
}

#[derive(Deserialize)]
pub struct SlimBadge {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: u32,
    pub description: Box<str>,
    #[serde(deserialize_with = "deserialize_users")]
    pub users: Box<[u32]>,
    pub image_url: Box<str>,
}

fn deserialize_id<'de, D: Deserializer<'de>>(d: D) -> Result<u32, D::Error> {
    let s = <&'de str as Deserialize>::deserialize(d)?;

    s.parse()
        .map_err(|_| SerdeError::invalid_value(Unexpected::Str(s), &"a u32"))
}

fn deserialize_users<'de, D: Deserializer<'de>>(d: D) -> Result<Box<[u32]>, D::Error> {
    let s = <&'de str as Deserialize>::deserialize(d)?;

    if s.is_empty() {
        return Ok(Box::default());
    }

    s[1..s.len() - 1]
        .split(',')
        .map(str::trim)
        .map(str::parse)
        .collect::<Result<_, _>>()
        .map(Vec::into_boxed_slice)
        .map_err(|_| {
            SerdeError::invalid_value(Unexpected::Str(s), &"a stringified list of integers")
        })
}

impl PartialEq for SlimBadge {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for SlimBadge {}

impl Ord for SlimBadge {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.description
            .cmp(&other.description)
            .then_with(|| self.image_url.cmp(&other.image_url))
    }
}

impl PartialOrd for SlimBadge {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
