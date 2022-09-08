use std::{
    collections::{HashMap, HashSet},
    fmt::{Formatter, Result as FmtResult},
    rc::Rc,
};

use eyre::{Context as _, Result};
use serde::{
    de::{DeserializeSeed, Error as SerdeError, SeqAccess, Unexpected, Visitor},
    Deserialize, Deserializer as _,
};
use serde_json::{de::SliceRead, Deserializer};

use crate::{model::Badge, util::IntHasher};

use super::Context;

impl Context {
    pub async fn gather_badges(&self) -> Result<HashMap<Rc<String>, Badge>> {
        todo!()
    }

    pub async fn gather_more_users(&self, users: &mut HashSet<u32, IntHasher>) -> Result<()> {
        let bytes = self
            .client
            .get_osekai_members()
            .await
            .context("failed to get osekai members")?;

        Deserializer::new(SliceRead::new(&bytes))
            .deserialize_seq(ExtendUsersVisitor(users))
            .with_context(|| {
                let text = String::from_utf8_lossy(&bytes);

                format!("failed to deserialize osekai members: {text}")
            })
    }

    pub async fn gather_rarities(&self) -> Result<HashMap<u32, f64, IntHasher>> {
        todo!()
    }
}

struct ExtendUsersVisitor<'u>(&'u mut HashSet<u32, IntHasher>);

impl<'de> Visitor<'de> for ExtendUsersVisitor<'_> {
    type Value = ();

    #[inline]
    fn expecting(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(r#"a list of `{"Id": "<number>"}` objects"#)
    }

    #[inline]
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        #[derive(Deserialize)]
        struct IdEntry<'s> {
            #[serde(rename = "Id")]
            id: &'s str,
        }

        while let Some(IdEntry { id }) = seq.next_element()? {
            let id = id
                .parse()
                .map_err(|_| SerdeError::invalid_value(Unexpected::Str(id), &"an integer"))?;

            self.0.insert(id);
        }

        Ok(())
    }
}
