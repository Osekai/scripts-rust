pub use self::hasher::{IntHash, IntHasher};

mod hasher;

#[cfg(debug_assertions)]
pub use self::generate::Generate;

#[cfg(debug_assertions)]
mod generate;
