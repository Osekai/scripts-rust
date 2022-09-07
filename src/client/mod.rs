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
    model::{Badge, MedalRarity, RankingUser, ScrapedMedal},
};

use self::{bytes::BodyBytes, multipart::Multipart};

mod bytes;
mod multipart;

static MY_USER_AGENT: &str = env!("CARGO_PKG_NAME");

type InnerClient = HyperClient<HttpsConnector<HttpConnector<GaiResolver>>, BodyBytes>;

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
        if cfg!(debug_assertion) {
            fs::read("./peppy.html").context("failed to read `./peppy.html`")
        } else {
            let url = "https://osu.ppy.sh/users/peppy/osu";

            let bytes = self
                .send_get_request(url)
                .await
                .context("failed to request user webpage")?;

            // fs::write("./peppy.html", &bytes).unwrap();

            Ok(bytes.into())
        }
    }

    pub async fn upload_medals(&self, medals: &[ScrapedMedal]) -> Result<Bytes> {
        let url = format!("{base}up_medals.php", base = Config::get().url_base);

        self.send_post_request(&url, &medals).await
    }

    pub async fn upload_rarity(&self, rarity: &[MedalRarity]) -> Result<Bytes> {
        let url = format!("{base}up_medals_rarity.php", base = Config::get().url_base);

        self.send_post_request(&url, &rarity).await
    }

    pub async fn upload_ranking(&self, ranking: &[RankingUser]) -> Result<Bytes> {
        let url = format!("{base}up_ranking.php", base = Config::get().url_base);

        self.send_post_request(&url, &ranking).await
    }

    pub async fn upload_badges(&self, badges: &[Badge]) -> Result<Bytes> {
        let url = format!("{base}up_badges.php", base = Config::get().url_base);

        self.send_post_request(&url, &badges).await
    }

    pub async fn finish_uploading(&self) -> Result<Bytes> {
        let url = format!("{base}finish.php", base = Config::get().url_base);

        self.send_get_request(&url).await
    }

    async fn send_get_request(&self, url: impl AsRef<str>) -> Result<Bytes> {
        let url = url.as_ref();
        trace!("sending GET request to url {url}");

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
        trace!("sending POST request to url {url}");

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
            !(status.is_client_error() || status.is_server_error()),
            "failed with status code {status} when requesting url {url}"
        );

        hyper::body::to_bytes(response.into_body())
            .await
            .context("failed to extract response bytes")
    }
}
