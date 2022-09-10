use std::fs;

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
    model::{Badges, MedalRarities, Progress, RankingUser, ScrapedMedal},
    task::Task,
};

use self::{bytes::BodyBytes, multipart::Multipart};

mod bytes;
mod multipart;

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

        let client = HyperClient::builder().build(connector);

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

    /// Request all badges stored by osekai
    pub async fn get_osekai_badges(&self) -> Result<Bytes> {
        let url = format!("{base}down_badges.php", base = Config::get().url_base);

        self.send_get_request(url).await
    }

    /// Request all user ids stored by osekai
    pub async fn get_osekai_members(&self) -> Result<Bytes> {
        let url = format!("{base}down_members.php", base = Config::get().url_base);

        self.send_get_request(url).await
    }

    /// Request all medal rarities stored by osekai
    pub async fn get_osekai_rarities(&self) -> Result<Bytes> {
        let base = &Config::get().url_base;
        let url = format!("{base}down_rarity.php");

        self.send_get_request(url).await
    }

    /// Upload medals to osekai
    pub async fn upload_medals(&self, medals: &[ScrapedMedal]) -> Result<Bytes> {
        let url = format!("{base}up_medals.php", base = Config::get().url_base);

        self.send_post_request(&url, &medals).await
    }

    /// Upload medal rarities to osekai
    pub async fn upload_rarity(&self, rarity: &MedalRarities) -> Result<Bytes> {
        let url = format!("{base}up_medals_rarity.php", base = Config::get().url_base);

        self.send_post_request(&url, rarity).await
    }

    /// Upload user rankings to osekai
    pub async fn upload_ranking(&self, ranking: &[RankingUser]) -> Result<Bytes> {
        let url = format!("{base}up_ranking.php", base = Config::get().url_base);

        self.send_post_request(&url, &ranking).await
    }

    /// Upload badges to osekai
    pub async fn upload_badges(&self, badges: &Badges) -> Result<Bytes> {
        let url = format!("{base}up_badges.php", base = Config::get().url_base);

        self.send_post_request(&url, badges).await
    }

    /// Keep osekai posted on what the current progress is
    pub async fn upload_progress(&self, progress: &Progress) -> Result<Bytes> {
        let url = format!("{base}progression.php", base = Config::get().url_base);

        self.send_post_request(&url, progress).await
    }

    /// Notify osekai that the upload iteration is finished
    pub async fn finish_uploading(&self, task: Task) -> Result<Bytes> {
        let url = format!("{base}finish.php", base = Config::get().url_base);

        self.send_post_request(&url, &task).await
    }

    async fn send_get_request(&self, url: impl AsRef<str>) -> Result<Bytes> {
        let url = url.as_ref();
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

    async fn send_post_request<J>(&self, url: impl AsRef<str>, data: &J) -> Result<Bytes>
    where
        J: Serialize,
    {
        let url = url.as_ref();
        trace!("Sending POST request to url {url}");

        let form = Multipart::new()
            .push_text("key", &Config::get().tokens.post)
            .push_json("data", data)
            .context("failed to push json onto multipart")?;

        let content_type = format!("multipart/form-data; boundary={}", form.boundary());
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
