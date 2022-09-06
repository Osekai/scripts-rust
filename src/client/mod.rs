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

use self::bytes::BodyBytes;

mod bytes;

static MY_USER_AGENT: &str = env!("CARGO_PKG_NAME");

type InnerClient = HyperClient<HttpsConnector<HttpConnector<GaiResolver>>, BodyBytes>;

pub struct Client {
    client: InnerClient,
    // ratelimiters: [LeakyBucket; 0],
}

impl Client {
    pub async fn new() -> Self {
        let connector = HttpsConnectorBuilder::new()
            .with_webpki_roots()
            .https_or_http()
            .enable_http1()
            .build();

        let client = HyperClient::builder().build(connector);

        Self { client }
    }

    /// Sends a GET request
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

    /// Sends a POST requesting containing JSON data
    async fn send_post_request<J>(&self, url: impl AsRef<str>, data: &J) -> Result<Bytes>
    where
        J: Serialize,
    {
        let url = url.as_ref();
        trace!("sending POST request to url {url}");

        let data = serde_json::to_vec(data).context("failed to serialize data")?;

        let req = Request::builder()
            .method(Method::POST)
            .uri(url)
            .header(USER_AGENT, MY_USER_AGENT)
            .header(CONTENT_TYPE, "application/json")
            .header(CONTENT_LENGTH, data.len())
            .body(data.into())
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
