pub use self::hasher::{IntHash, IntHasher};

mod hasher;

#[cfg(any(debug_assertions, feature = "generate"))]
pub use self::generate::Generate;

#[cfg(any(debug_assertions, feature = "generate"))]
mod generate;
