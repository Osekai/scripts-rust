use std::{fs, time::Duration};

use ::bytes::Bytes;
use eyre::{Context as _, Result};
use http::{HeaderValue, Uri};
use hyper::{
    client::{connect::dns::GaiResolver, Client as HyperClient, HttpConnector},
    header::{CONTENT_TYPE, USER_AGENT},
    Method, Request,
};
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use serde::Serialize;
use serde_json::{Error as JsonError, Serializer as JsonSerializer};

use crate::{
    config::Config,
    model::{Finish, Progress},
};

use self::bytes::BodyBytes;

mod bytes;

static MY_USER_AGENT: HeaderValue = HeaderValue::from_static(env!("CARGO_PKG_NAME"));
static FORM_URLENCODED: HeaderValue = HeaderValue::from_static("application/x-www-form-urlencoded");

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
            let url = Uri::from_static("https://osu.ppy.sh/users/2/osu");

            let bytes = self
                .send_get_request(&url)
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

    async fn send_get_request(&self, url: &Uri) -> Result<Bytes> {
        trace!("Sending GET request to url {url}");

        let req = Request::builder()
            .uri(url)
            .method(Method::GET)
            .header(USER_AGENT, &MY_USER_AGENT)
            .body(BodyBytes::default())
            .context("failed to build GET request")?;

        let response = self
            .client
            .request(req)
            .await
            .context("failed to send GET request")?;

        let status = response.status();

        ensure!(
            status.is_success(),
            "failed with status code {status} when requesting url {url}"
        );

        hyper::body::to_bytes(response.into_body())
            .await
            .context("failed to extract response bytes")
    }

    async fn notify_webhook(&self, content: String) -> Result<()> {
        trace!("Sending POST request for webhook");

        #[derive(Serialize)]
        struct UrlEncode {
            content: String,
        }

        let encode = UrlEncode { content };

        let encoded = serde_urlencoded::to_string(&encode)
            .context("failed to urlencode webhook notification")?;

        let req = Request::builder()
            .method(Method::GET)
            .uri(&Config::get().webhook_url)
            .header(USER_AGENT, &MY_USER_AGENT)
            .header(CONTENT_TYPE, &FORM_URLENCODED)
            .body(BodyBytes::from(encoded.into_bytes()))
            .context("failed to build POST request")?;

        let response = self
            .client
            .request(req)
            .await
            .context("failed to send POST request")?;

        let status = response.status();

        ensure!(
            status.is_success(),
            "failed with status code {status} when notifying webhook"
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
