use std::{fs, time::Duration};

use ::bytes::Bytes;
use eyre::{Context as _, Result};
use http_body_util::{BodyExt, Collected, Full};
use hyper::{
    header::{HeaderValue, CONTENT_LENGTH, CONTENT_TYPE, USER_AGENT},
    Request, Uri,
};
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use hyper_util::{
    client::legacy::{connect::HttpConnector, Builder, Client as HyperClient},
    rt::TokioExecutor,
};
use serde::Serialize;
use serde_json::{Error as JsonError, Serializer as JsonSerializer};

use crate::{
    config::Config,
    model::{Finish, Progress},
};

static MY_USER_AGENT: HeaderValue = HeaderValue::from_static(concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION")
));
static FORM_URLENCODED: HeaderValue = HeaderValue::from_static("application/x-www-form-urlencoded");

type Body = Full<Bytes>;

/// Client that makes all requests that do not go to the osu!api itself
pub struct Client {
    client: HyperClient<HttpsConnector<HttpConnector>, Body>,
}

impl Client {
    pub fn new() -> Self {
        let crypto_provider = rustls::crypto::ring::default_provider();

        let https = HttpsConnectorBuilder::new()
            .with_provider_and_webpki_roots(crypto_provider)
            .expect("Failed to configure https connector")
            .https_only()
            .enable_http2()
            .build();

        let client = Builder::new(TokioExecutor::new())
            .pool_idle_timeout(Duration::from_secs(15)) // https://github.com/hyperium/hyper/issues/2136
            .http2_only(true)
            .build(https);

        Self { client }
    }

    /// Requests peppy's webpage and returns its bytes
    pub async fn get_user_webpage(&self) -> Result<Vec<u8>> {
        // Avoid request spamming while debugging
        if cfg!(debug_assertions) {
            debug!("Reading ./peppy.html instead of requesting the webpage");

            fs::read("./peppy.html").context("failed to read `./peppy.html`")
        } else {
            let url = Uri::from_static("https://osu.ppy.sh/users/2/osu");

            let bytes = self
                .send_get_request(url)
                .await
                .context("failed to request user webpage")?;

            // fs::write("./peppy.html", &bytes).unwrap();

            Ok(bytes.into())
        }
    }

    /// Keep a webhook posted on what the current progress is
    pub async fn notify_webhook_progress(&self, progress: &Progress) -> Result<()> {
        const CONTENT_PREFIX: &str = "SCRIPTS-RUST Progress Update:\n";

        let content = serialize_webhook_content(CONTENT_PREFIX, progress)
            .context("failed to serialize progress to json")?;

        self.notify_webhook(content).await
    }

    /// Notify a webhook that the upload iteration is finished
    pub async fn notify_webhook_finish(&self, finish: &Finish) -> Result<()> {
        const CONTENT_PREFIX: &str = "scripts-rust upload<br>";

        let content = serialize_webhook_content(CONTENT_PREFIX, finish)
            .context("failed to serialize finish to json")?;

        self.notify_webhook(content).await
    }

    async fn send_get_request(&self, url: Uri) -> Result<Bytes> {
        trace!("Sending GET request to url {url}");

        let req = Request::get(&url)
            .header(USER_AGENT, &MY_USER_AGENT)
            .body(Body::default())
            .context("failed to create GET request")?;

        let response = self
            .client
            .request(req)
            .await
            .context("failed to fetch GET response")?;

        let status = response.status();

        ensure!(
            status.is_success(),
            "failed with status code {status} when requesting url {url}"
        );

        response
            .into_body()
            .collect()
            .await
            .context("failed to collect response bytes")
            .map(Collected::to_bytes)
    }

    async fn notify_webhook(&self, content: String) -> Result<()> {
        trace!("Sending POST request for webhook");

        #[derive(Serialize)]
        struct UrlEncode {
            content: String,
        }

        let encode = UrlEncode { content };

        let body = serde_urlencoded::to_string(&encode)
            .context("failed to urlencode webhook notification")?
            .into_bytes();

        let req = Request::post(&Config::get().webhook_url)
            .header(USER_AGENT, &MY_USER_AGENT)
            .header(CONTENT_TYPE, &FORM_URLENCODED)
            .header(CONTENT_LENGTH, body.len())
            .body(Full::from(body))
            .context("failed to build POST request")?;

        let response = self
            .client
            .request(req)
            .await
            .context("failed to create POST request")?;

        let status = response.status();

        ensure!(
            status.is_success(),
            "failed to notify webhook status={status}"
        );

        Ok(())
    }
}

fn serialize_webhook_content<T: Serialize>(prefix: &str, data: &T) -> Result<String, JsonError> {
    let mut content = String::with_capacity(128);
    content.push_str(prefix);

    // SAFETY: serde_json does not emit invalid UTF-8
    let content_bytes = unsafe { content.as_mut_vec() };

    let mut serializer = JsonSerializer::new(content_bytes);
    data.serialize(&mut serializer)?;

    Ok(content)
}
