use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
    hash::Hash,
};

use bytes::Bytes;
use eyre::{Context as _, Report, Result};
use http::{Response, StatusCode};
use hyper::{
    client::{connect::dns::GaiResolver, Client as HyperClient, HttpConnector},
    header::{CONTENT_TYPE, USER_AGENT},
    Body, Method, Request,
};
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use leaky_bucket_lite::LeakyBucket;
use serde::Serialize;
use tokio::time::{sleep, Duration};
use twilight_model::channel::Attachment;

use crate::util::{constants::OSU_BASE, ExponentialBackoff};

mod deserialize;

static MY_USER_AGENT: &str = env!("CARGO_PKG_NAME");

const APPLICATION_JSON: &str = "application/json";
const APPLICATION_URLENCODED: &str = "application/x-www-form-urlencoded";

#[derive(Copy, Clone, Eq, Hash, PartialEq)]
#[repr(u8)]
enum Site {
    DiscordAttachment,
    DownloadChimu,
    DownloadKitsu,
}

type Client = HyperClient<HttpsConnector<HttpConnector<GaiResolver>>, Body>;

pub struct CustomClient {
    client: Client,
    ratelimiters: [LeakyBucket; 3],
}

impl CustomClient {
    pub fn new() -> Self {
        let connector = HttpsConnectorBuilder::new()
            .with_webpki_roots()
            .https_or_http()
            .enable_http1()
            .build();

        let client = HyperClient::builder().build(connector);

        let ratelimiter = |per_second| {
            LeakyBucket::builder()
                .max(per_second)
                .tokens(per_second)
                .refill_interval(Duration::from_millis(1000 / per_second as u64))
                .refill_amount(1)
                .build()
        };

        let ratelimiters = [
            ratelimiter(2), // DiscordAttachment
            ratelimiter(1), // DownloadChimu
            ratelimiter(1), // DownloadKitsu
        ];

        Self {
            client,
            ratelimiters,
        }
    }

    async fn ratelimit(&self, site: Site) {
        self.ratelimiters[site as usize].acquire_one().await
    }

    async fn make_get_request(&self, url: impl AsRef<str>, site: Site) -> Result<Bytes> {
        trace!("GET request of url {}", url.as_ref());

        let req = Request::builder()
            .uri(url.as_ref())
            .method(Method::GET)
            .header(USER_AGENT, MY_USER_AGENT)
            .body(Body::empty())
            .context("failed to build GET request")?;

        self.ratelimit(site).await;

        let response = self
            .client
            .request(req)
            .await
            .context("failed to receive GET response")?;

        Self::error_for_status(response, url.as_ref()).await
    }

    async fn make_post_request<F: Serialize>(
        &self,
        url: impl AsRef<str>,
        site: Site,
        form: &F,
    ) -> Result<Bytes> {
        trace!("POST request of url {}", url.as_ref());

        let form_body = serde_urlencoded::to_string(form)?;

        let req = Request::builder()
            .method(Method::POST)
            .uri(url.as_ref())
            .header(USER_AGENT, MY_USER_AGENT)
            .header(CONTENT_TYPE, APPLICATION_URLENCODED)
            .body(Body::from(form_body))
            .context("failed to build POST request")?;

        self.ratelimit(site).await;

        let response = self
            .client
            .request(req)
            .await
            .context("failed to receive POST response")?;

        Self::error_for_status(response, url.as_ref()).await
    }

    async fn error_for_status(response: Response<Body>, url: &str) -> Result<Bytes> {
        let status = response.status();

        if status.is_client_error() || status.is_server_error() {
            bail!("failed with status code {status} when requesting {url}")
        } else {
            let bytes = hyper::body::to_bytes(response.into_body())
                .await
                .context("failed to extract response bytes")?;

            Ok(bytes)
        }
    }

    pub async fn get_discord_attachment(&self, attachment: &Attachment) -> Result<Bytes> {
        self.make_get_request(&attachment.url, Site::DiscordAttachment)
            .await
    }

    pub async fn download_chimu_mapset(&self, mapset_id: u32) -> Result<Bytes> {
        let url = format!("https://chimu.moe/d/{mapset_id}");

        self.make_get_request(url, Site::DownloadChimu).await
    }

    pub async fn download_kitsu_mapset(&self, mapset_id: u32) -> Result<Bytes> {
        let url = format!("https://kitsu.moe/api/d/{mapset_id}");

        self.make_get_request(url, Site::DownloadKitsu).await
    }
}
