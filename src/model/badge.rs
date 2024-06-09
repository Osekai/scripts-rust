use std::{
    borrow::{Borrow, Cow},
    cmp::Ordering,
    collections::{hash_map::Entry, HashMap, HashSet},
    hash::{Hash, Hasher},
    mem,
};

use rosu_v2::prelude::Badge;
use time::OffsetDateTime;

use crate::util::IntHasher;

#[derive(PartialEq, Eq, Hash)]
pub struct BadgeName(pub Box<str>);

impl Borrow<str> for BadgeName {
    fn borrow(&self) -> &str {
        self.0.as_ref()
    }
}

pub struct BadgeImageUrl(pub Box<str>);

#[derive(PartialEq, Eq, Hash)]
pub struct BadgeDescription(pub Box<str>);

impl Borrow<str> for BadgeDescription {
    fn borrow(&self) -> &str {
        self.0.as_ref()
    }
}

pub struct BadgeOwner {
    pub user_id: u32,
    pub awarded_at: OffsetDateTime,
}

impl PartialEq for BadgeOwner {
    fn eq(&self, other: &Self) -> bool {
        self.user_id == other.user_id
    }
}

impl Eq for BadgeOwner {}

impl Hash for BadgeOwner {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.user_id.hash(state);
    }
}

type BadgeOwners = HashSet<BadgeOwner, IntHasher>;

struct PendingImageUrl<'a>(Cow<'a, str>);

impl PendingImageUrl<'_> {
    /// Returns `Err` if there is no `/` or `.` after the `/`.
    fn name(&self, buf: &mut String) -> Result<(), ()> {
        let (name_raw, _) = self
            .0
            .rsplit_once('/')
            .and_then(|(_, file)| file.rsplit_once('.'))
            .ok_or(())?;

        buf.clear();

        let chars = name_raw.chars().map(|ch| match ch {
            '-' | '_' => ' ',
            _ => ch,
        });

        buf.extend(chars);

        Ok(())
    }
}

impl<'a> From<PendingImageUrl<'a>> for BadgeImageUrl {
    fn from(pending: PendingImageUrl<'a>) -> Self {
        let image_url = match pending.0 {
            Cow::Borrowed(image_url) => Box::from(image_url),
            Cow::Owned(image_url) => image_url.into_boxed_str(),
        };

        Self(image_url)
    }
}

#[derive(Default)]
pub struct Badges {
    pub names: HashMap<BadgeName, BadgeImageUrl>,
    /// Different badges might have the same description but owners of the same
    /// badge don't necessarily have the same description.
    pub descriptions: HashMap<BadgeDescription, HashMap<BadgeName, BadgeOwners>>,
}

impl Badges {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            names: HashMap::with_capacity(capacity),
            descriptions: HashMap::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, user_id: u32, original: &mut Badge, name_buf: &mut String) {
        // Extract the image url.
        let image_url = match original.image_url.split_once('?') {
            Some((image_url, _)) => PendingImageUrl(Cow::Borrowed(image_url)),
            None => PendingImageUrl(Cow::Owned(mem::take(&mut original.image_url))),
        };

        // Extract the name from the image url.
        if image_url.name(name_buf).is_err() {
            warn!(
                "Invalid name for badge with image url `{}`",
                original.image_url
            );

            return;
        }

        // If it's a new name, add it as entry.
        if !self.names.contains_key(name_buf.as_str()) {
            let name = Box::from(name_buf.as_str());
            self.names
                .insert(BadgeName(name), BadgeImageUrl::from(image_url));
        }

        let owner = BadgeOwner {
            user_id,
            awarded_at: original.awarded_at,
        };

        if let Some(entries) = self.descriptions.get_mut(original.description.as_str()) {
            // The description already has an entry.
            if let Some(owners) = entries.get_mut(name_buf.as_str()) {
                // The name already has an entry.
                owners.insert(owner);
            } else {
                // Seeing the name for that description for the first time.
                // Adding new entry.
                let mut owners = HashSet::default();
                owners.insert(owner);
                let name = BadgeName(Box::from(name_buf.as_str()));
                entries.insert(name, owners);
            }
        } else {
            // Seeing that description for the first time. Adding new entry.
            let mut owners = HashSet::default();
            owners.insert(owner);
            let name = BadgeName(Box::from(name_buf.as_str()));
            let mut entries = HashMap::default();
            entries.insert(name, owners);
            let description =
                BadgeDescription(mem::take(&mut original.description).into_boxed_str());
            self.descriptions.insert(description, entries);
        }
    }

    pub fn len(&self) -> usize {
        self.descriptions
            .values()
            .flat_map(HashMap::values)
            .map(HashSet::len)
            .sum()
    }

    pub fn is_empty(&self) -> bool {
        self.descriptions.is_empty()
    }

    pub fn merge(&mut self, mut other: Self) {
        self.names.extend(other.names.drain());

        for (description, entries) in other.descriptions.drain() {
            match self.descriptions.entry(description) {
                Entry::Occupied(entry) => {
                    let this = entry.into_mut();

                    for (name, owners) in entries {
                        match this.entry(name) {
                            Entry::Occupied(entry) => entry.into_mut().extend(owners),
                            Entry::Vacant(entry) => {
                                entry.insert(owners);
                            }
                        }
                    }
                }
                Entry::Vacant(entry) => {
                    entry.insert(entries);
                }
            }
        }
    }
}

pub struct SlimBadge {
    pub id: u32,
    pub description: Box<str>,
    pub users: Box<[u32]>,
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
