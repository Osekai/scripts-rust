use std::{env, path::PathBuf};

use eyre::{Context as _, ContextCompat as _, Result};
use once_cell::sync::OnceCell;

static CONFIG: OnceCell<Config> = OnceCell::new();

pub struct Config {
    pub tokens: Tokens,
}

pub struct Tokens {
    pub post: String,
}

impl Config {
    pub fn get() -> &'static Self {
        unsafe { CONFIG.get_unchecked() }
    }
}

pub fn init() -> Result<()> {
    let config = Config {
        tokens: Tokens {
            post: env_var("POST_KEY")?,
        },
    };

    CONFIG
        .set(config)
        .map_err(|_| eyre!("`Config::init` has already been called"))
}

trait EnvKind: Sized {
    const EXPECTED: &'static str;

    fn from_str(s: String) -> Result<Self, String>;
}

macro_rules! env_kind {
    ($($ty:ty: $arg:ident => $impl:block,)*) => {
        $(
            impl EnvKind for $ty {
                const EXPECTED: &'static str = stringify!($ty);

                fn from_str($arg: String) -> Result<Self, String> {
                    $impl
                }
            }
        )*
    };
}

env_kind! {
    u16: s => { s.parse().map_err(|_| s) },
    u64: s => { s.parse().map_err(|_| s) },
    PathBuf: s => { s.parse().map_err(|_| s) },
    String: s => { Ok(s) },
}

fn env_var<T: EnvKind>(name: &'static str) -> Result<T> {
    let value = env::var(name).with_context(|| format!("missing env variable `{name}`"))?;

    T::from_str(value).map_err(|value| {
        eyre!(
            "failed to parse env variable `{name}={value}`; expected {expected}",
            expected = T::EXPECTED
        )
    })
}
