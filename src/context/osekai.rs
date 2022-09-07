use std::collections::HashMap;

use super::Context;

use eyre::Result;

impl Context {
    pub async fn gather_badges(&self) -> Result<Vec<()>> {
        todo!()
    }

    pub async fn gather_users(&self) -> Result<Vec<u32>> {
        todo!()
    }

    pub async fn gather_rarities(&self) -> Result<HashMap<u32, f64>> {
        todo!()
    }
}
