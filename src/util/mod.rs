pub use self::{
    args::Args,
    eta::{Eta, TimeEstimate},
    hasher::{IntHash, IntHasher},
};

mod args;
mod eta;
mod hasher;

#[cfg(feature = "generate")]
pub use self::generate::{Generate, GenerateRange};

#[cfg(feature = "generate")]
mod generate;
