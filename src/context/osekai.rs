use std::collections::{HashMap, HashSet};

use crate::model::Badge;

use super::Context;

use eyre::Result;

impl Context {
    pub async fn gather_badges(&self) -> Result<Vec<Badge>> {
        todo!()
    }

    pub async fn gather_more_users(&self, users: &mut HashSet<u32>) -> Result<()> {
        todo!()
    }

    pub async fn gather_rarities(&self) -> Result<HashMap<u32, f64>> {
        todo!()
    }
}
