use std::{env, sync::OnceLock};

use eyre::{Context as _, Result};
use http::Uri;

use crate::{schedule::Schedule, util::Args};

static CONFIG: OnceLock<Config> = OnceLock::new();

pub struct Config {
    pub tokens: Tokens,
    pub database_url: Box<str>,
    pub webhook_url: Uri,
    pub schedule: Schedule,
}

pub struct Tokens {
    pub post: Box<str>,
    pub osu_client_id: u64,
    pub osu_client_secret: Box<str>,
}

impl Config {
    pub fn get() -> &'static Self {
        CONFIG.get().expect("CONFIG not yet initialized")
    }
}

pub fn init(args: &mut Args) -> Result<()> {
    let extra_users = env::var("EXTRA_USERS").unwrap_or_else(|_| {
        warn!(
            "missing env variable `EXTRA_USERS`; \
            will consider this as no extra users"
        );

        String::new()
    });

    let extra_users_iter = extra_users
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::parse);

    for extra_user in extra_users_iter {
        let user = extra_user.with_context(|| {
            format!(
                "failed to parse env variable `EXTRA_USERS=\"{extra_users}\"`; \
                expected a list of comma-separated user ids"
            )
        })?;

        args.extras.insert(user);
    }

    let config = Config {
        tokens: Tokens {
            post: env_var("POST_KEY")?,
            osu_client_id: env_var("OSU_CLIENT_ID")?,
            osu_client_secret: env_var("OSU_CLIENT_SECRET")?,
        },
        database_url: env_var("DATABASE_URL")?,
        webhook_url: env_var("WEBHOOK_URL")?,
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
    Box<str>: s => { Ok(s.into_boxed_str()) },
    u64: s => { s.parse().map_err(|_| s) },
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
