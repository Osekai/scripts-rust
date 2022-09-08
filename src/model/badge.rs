use std::{
    collections::HashMap,
    fmt::{Display, Formatter, Result as FmtResult},
    mem,
};

use rosu_v2::prelude::Badge;
use serde::{
    ser::{SerializeMap, SerializeSeq},
    Serialize, Serializer,
};
use time::OffsetDateTime;

pub struct BadgeEntry {
    pub awarded_at: OffsetDateTime,
    pub users: Vec<u32>,
    pub image_url: String,
    pub url: String,
}

#[derive(Default)]
pub struct Badges {
    inner: HashMap<String, BadgeEntry>,
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
        let key = mem::take(&mut badge.description);

        let entry = self.inner.entry(key).or_insert_with(|| BadgeEntry {
            awarded_at: badge.awarded_at,
            users: Vec::new(),
            image_url: mem::take(&mut badge.image_url),
            url: mem::take(&mut badge.url),
        });

        entry.users.push(user_id);
    }
}

impl Serialize for Badges {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut s = s.serialize_seq(Some(self.inner.len()))?;

        for (key, value) in self.inner.iter() {
            let entry = BorrowedBadge::new(key, value);
            s.serialize_element(&entry)?;
        }

        s.end()
    }
}

struct BorrowedBadge<'b> {
    description: &'b String,
    badge: &'b BadgeEntry,
}

impl<'b> BorrowedBadge<'b> {
    fn new(description: &'b String, badge: &'b BadgeEntry) -> Self {
        Self { description, badge }
    }
}

impl Serialize for BorrowedBadge<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut s = s.serialize_map(Some(5))?;

        s.serialize_entry("awarded_at", &Date(self.badge.awarded_at))?;
        s.serialize_entry("description", &self.description)?;
        s.serialize_entry("users", &self.badge.users)?;
        s.serialize_entry("image_url", &self.badge.image_url)?;
        s.serialize_entry("url", &self.badge.url)?;

        s.end()
    }
}

struct Date(OffsetDateTime);

impl Display for Date {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let date = self.0.date();

        write!(f, "{}-{}-{}", date.year(), date.month(), date.day())
    }
}

impl Serialize for Date {
    #[inline]
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}
