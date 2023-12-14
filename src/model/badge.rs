use std::{
    borrow::Borrow,
    cmp::Ordering,
    collections::{
        hash_map::{Iter, IterMut},
        HashMap,
    },
    mem,
};

use rosu_v2::prelude::Badge;
use time::Date;

pub type BadgeAwards = Vec<(BadgeId, UserId, Date)>;
pub type BadgeId = u32;
pub type UserId = u32;

// Different badges may have the same description so we
// use the image url as key instead.
//
// See github issue #1
#[derive(Eq, PartialEq, Hash)]
pub struct BadgeKey {
    pub image_url: Box<str>,
}

impl Borrow<str> for BadgeKey {
    fn borrow(&self) -> &str {
        &self.image_url
    }
}

pub struct BadgeEntry {
    pub description: Box<str>,
    pub id: Option<u32>,
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

    pub fn insert(&mut self, badge: &mut Badge) {
        if let Some(idx) = badge.image_url.find('?') {
            badge.image_url.truncate(idx);
        }

        let image_url = badge.image_url.as_str();

        if self.inner.contains_key(image_url) {
            return;
        }

        let key = BadgeKey {
            // We'll need the image url again later so instead of
            // `mem::take` we just clone it.
            image_url: Box::from(image_url),
        };

        let value = BadgeEntry {
            description: mem::take(&mut badge.description).into_boxed_str(),
            id: None,
        };

        self.inner.insert(key, value);
    }

    pub fn iter(&self) -> Iter<'_, BadgeKey, BadgeEntry> {
        self.inner.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, BadgeKey, BadgeEntry> {
        self.inner.iter_mut()
    }
}

#[derive(Debug)]
pub struct SlimBadge {
    pub id: u32,
    pub description: Box<str>,
    pub image_url: Box<str>,
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
