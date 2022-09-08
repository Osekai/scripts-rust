pub use self::hasher::{IntHash, IntHasher};

mod hasher;

#[cfg(feature = "generate")]
pub use self::generate::Generate;

#[cfg(feature = "generate")]
mod generate;
