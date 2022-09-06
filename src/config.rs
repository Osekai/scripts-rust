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

    pub fn init() -> Result<()> {
        let config = Config {
            tokens: Tokens {
                post: env_var("POST_KEY")?,
            },
        };

        if CONFIG.set(config).is_err() {
            error!("`Config::init` has already been called");
        }

        Ok(())
    }
}

trait EnvKind: Sized {
    const EXPECTED: &'static str;

    fn from_str(s: &str) -> Option<Self>;
}

macro_rules! env_kind {
    ($($ty:ty: $arg:ident => $impl:block,)*) => {
        $(
            impl EnvKind for $ty {
                const EXPECTED: &'static str = stringify!($ty);

                fn from_str($arg: &str) -> Option<Self> {
                    $impl
                }
            }
        )*
    };
}

env_kind! {
    u16: s => { s.parse().ok() },
    u64: s => { s.parse().ok() },
    PathBuf: s => { s.parse().ok() },
    String: s => { Some(s.to_owned()) },
}

fn env_var<T: EnvKind>(name: &'static str) -> Result<T> {
    let value = env::var(name).with_context(|| format!("missing env variable `{name}`"))?;

    T::from_str(&value).with_context(|| {
        format!(
            "failed to parse env variable `{name}={value}`; expected {expected}",
            expected = T::EXPECTED
        )
    })
}
