use std::{env, path::PathBuf};

use eyre::{Context as _, Result};
use http::Uri;
use once_cell::sync::OnceCell;

use crate::schedule::Schedule;

static CONFIG: OnceCell<Config> = OnceCell::new();

pub struct Config {
    pub tokens: Tokens,
    pub url_base: Uri,
    pub schedule: Schedule,
}

pub struct Tokens {
    pub post: String,
    pub osu_client_id: u64,
    pub osu_client_secret: String,
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
            osu_client_id: env_var("OSU_CLIENT_ID")?,
            osu_client_secret: env_var("OSU_CLIENT_SECRET")?,
        },
        url_base: env_var("URL_BASE")?,
        schedule: env::var("SCHEDULE")
            .map_err(|_| eyre!("missing env variable `SCHEDULE`"))?
            .parse()
            .context("failed to parse schedule; must be a comma-separated list of tasks")?,
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
    String: s => { Ok(s) },
    u64: s => { s.parse().map_err(|_| s) },
    PathBuf: s => { s.parse().map_err(|_| s) },
    Uri: s => { s.parse().map_err(|_| s) },
}

fn env_var<T: EnvKind>(name: &'static str) -> Result<T> {
    let value = env::var(name).map_err(|_| eyre!("missing env variable `{name}`"))?;

    T::from_str(value).map_err(|value| {
        eyre!(
            "failed to parse env variable `{name}={value}`; expected {expected}",
            expected = T::EXPECTED
        )
    })
}
