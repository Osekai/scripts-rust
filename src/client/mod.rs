use std::{fs, time::Duration};

use ::bytes::Bytes;
use eyre::{Context as _, Result};
use http::{header::CONTENT_LENGTH, Response};
use hyper::{
    client::{connect::dns::GaiResolver, Client as HyperClient, HttpConnector},
    header::{CONTENT_TYPE, USER_AGENT},
    Body, Method, Request,
};
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use serde::Serialize;

use crate::{
    config::Config,
    model::{Badges, Finish, MedalRarities, Progress, RankingUser, ScrapedMedal},
};

use self::{bytes::BodyBytes, multipart::Multipart};
pub use response::OsekaiResponse;

mod bytes;
mod multipart;
mod response;

static MY_USER_AGENT: &str = env!("CARGO_PKG_NAME");

type InnerClient = HyperClient<HttpsConnector<HttpConnector<GaiResolver>>, BodyBytes>;

/// Client that makes all requests that do not go to the osu!api itself
pub struct Client {
    client: InnerClient,
}

impl Client {
    pub fn new() -> Self {
        let connector = HttpsConnectorBuilder::new()
            .with_webpki_roots()
            .https_or_http()
            .enable_http1()
            .build();

        let client = HyperClient::builder()
            .pool_idle_timeout(Duration::from_secs(20)) // https://github.com/hyperium/hyper/issues/2136
            .build(connector);

        Self { client }
    }

    /// Requests peppy's webpage and returns its bytes
    pub async fn get_user_webpage(&self) -> Result<Vec<u8>> {
        // Avoid request spamming while debugging
        if cfg!(debug_assertions) {
            debug!("Reading ./peppy.html instead of requesting the webpage");

            fs::read("./peppy.html").context("failed to read `./peppy.html`")
        } else {
            let url = "https://osu.ppy.sh/users/2/osu";

            let bytes = self
                .send_get_request(url)
                .await
                .context("failed to request user webpage")?;

            // fs::write("./peppy.html", &bytes).unwrap();

            Ok(bytes.into())
        }
    }

    /// Request all medals stored by osekai
    pub async fn get_osekai_medals(&self) -> Result<Bytes> {
        let url = format!("{base}down_medals.php", base = Config::get().url_base);

        self.send_get_request_retry(url).await
    }

    /// Request all badges stored by osekai
    pub async fn get_osekai_badges(&self) -> Result<Bytes> {
        let url = format!("{base}down_badges.php", base = Config::get().url_base);

        self.send_get_request_retry(url).await
    }

    /// Request all user ids stored by osekai
    pub async fn get_osekai_members(&self) -> Result<Bytes> {
        let url = format!("{base}down_members.php", base = Config::get().url_base);

        self.send_get_request_retry(url).await
    }

    /// Request all ranking user ids stored by osekai
    pub async fn get_osekai_ranking(&self) -> Result<Bytes> {
        let url = format!("{base}down_ranking_ids.php", base = Config::get().url_base);

        self.send_get_request_retry(url).await
    }

    /// Request all medal rarities stored by osekai
    pub async fn get_osekai_rarities(&self) -> Result<Bytes> {
        let base = &Config::get().url_base;
        let url = format!("{base}down_rarity.php");

        self.send_get_request_retry(url).await
    }

    /// Upload medals to osekai
    pub async fn upload_medals(&self, medals: &[ScrapedMedal]) -> Result<OsekaiResponse> {
        let url = format!("{base}up_medals.php", base = Config::get().url_base);
        let bytes = self.send_post_request_retry(&url, &medals).await?;

        OsekaiResponse::new(bytes)
    }

    /// Upload medal rarities to osekai
    pub async fn upload_rarity(&self, rarity: &MedalRarities) -> Result<OsekaiResponse> {
        let url = format!("{base}up_medals_rarity.php", base = Config::get().url_base);
        let bytes = self.send_post_request_retry(&url, rarity).await?;

        OsekaiResponse::new(bytes)
    }

    /// Upload user rankings to osekai
    pub async fn upload_ranking(&self, ranking: &[RankingUser]) -> Result<OsekaiResponse> {
        let url = format!("{base}up_ranking.php", base = Config::get().url_base);
        let bytes = self.send_post_request_retry(&url, &ranking).await?;

        OsekaiResponse::new(bytes)
    }

    /// Upload badges to osekai
    pub async fn upload_badges(&self, badges: &Badges) -> Result<OsekaiResponse> {
        let url = format!("{base}up_badges.php", base = Config::get().url_base);
        let bytes = self.send_post_request_retry(&url, badges).await?;

        OsekaiResponse::new(bytes)
    }

    /// Keep osekai posted on what the current progress is
    pub async fn upload_progress(&self, progress: &Progress) -> Result<OsekaiResponse> {
        let url = format!("{base}progression.php", base = Config::get().url_base);
        let bytes = self.send_post_request_retry(&url, progress).await?;

        OsekaiResponse::new(bytes)
    }

    /// Notify osekai that the upload iteration is finished
    pub async fn finish_uploading(&self, finish: Finish) -> Result<OsekaiResponse> {
        let url = format!("{base}finish.php", base = Config::get().url_base);
        let bytes = self.send_post_request_retry(&url, &finish).await?;

        OsekaiResponse::new(bytes)
    }

    async fn send_get_request_retry(&self, url: impl AsRef<str>) -> Result<Bytes> {
        let url = url.as_ref();

        match self.send_get_request(url).await {
            Ok(bytes) => Ok(bytes),
            Err(_) => self.send_get_request(url).await,
        }
    }

    async fn send_get_request(&self, url: &str) -> Result<Bytes> {
        trace!("Sending GET request to url {url}");

        let req = Request::builder()
            .uri(url)
            .method(Method::GET)
            .header(USER_AGENT, MY_USER_AGENT)
            .body(BodyBytes::default())
            .context("failed to build GET request")?;

        let response = self
            .client
            .request(req)
            .await
            .context("failed to receive GET response")?;

        Self::error_for_status(response, url).await
    }

    async fn send_post_request_retry<J>(&self, url: impl AsRef<str>, data: &J) -> Result<Bytes>
    where
        J: Serialize,
    {
        let url = url.as_ref();

        match self.send_post_request(url, data).await {
            Ok(bytes) => Ok(bytes),
            Err(_) => self.send_post_request(url, data).await,
        }
    }

    #[cfg_attr(debug_assertions, allow(unused))]
    async fn send_post_request<J>(&self, url: &str, data: &J) -> Result<Bytes>
    where
        J: Serialize,
    {
        trace!("Sending POST request to url {url}");

        #[cfg(debug_assertions)]
        return Ok(Bytes::new());

        let form = Multipart::new()
            .push_text("key", Config::get().tokens.post.as_ref())
            .push_json("data", data)
            .context("failed to push json onto multipart")?;

        let content_type = form.content_type();
        let form = BodyBytes::from(form);

        let req = Request::builder()
            .method(Method::POST)
            .uri(url)
            .header(USER_AGENT, MY_USER_AGENT)
            .header(CONTENT_TYPE, content_type)
            .header(CONTENT_LENGTH, form.len())
            .body(form)
            .context("failed to build POST request")?;

        let response = self
            .client
            .request(req)
            .await
            .context("failed to receive POST response")?;

        Self::error_for_status(response, url).await
    }

    async fn error_for_status(response: Response<Body>, url: &str) -> Result<Bytes> {
        let status = response.status();

        ensure!(
            status.is_success(),
            "failed with status code {status} when requesting url {url}"
        );

        hyper::body::to_bytes(response.into_body())
            .await
            .context("failed to extract response bytes")
    }
}
