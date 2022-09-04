use std::{ffi::OsStr, hash::Hash, io::Write as _, path::Path};

use bytes::Bytes;
use eyre::{Context as _, ContextCompat as _, Result};
use http::Response;
use hyper::{
    client::{connect::dns::GaiResolver, Client as HyperClient, HttpConnector},
    header::{CONTENT_TYPE, USER_AGENT},
    Body, Method, Request,
};
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use leaky_bucket_lite::LeakyBucket;
use serde::{Deserialize, Serialize};
use tokio::{
    fs::{self, File},
    io::AsyncReadExt,
    time::Duration,
};
use twilight_model::channel::Attachment;

use crate::core::BotConfig;

mod deserialize;

static MY_USER_AGENT: &str = env!("CARGO_PKG_NAME");

const APPLICATION_URLENCODED: &str = "application/x-www-form-urlencoded";

#[derive(Copy, Clone, Eq, Hash, PartialEq)]
#[repr(u8)]
enum Site {
    DiscordAttachment,
    DownloadChimu,
    DownloadKitsu,
    ShishaMezo,
}

type Client = HyperClient<HttpsConnector<HttpConnector<GaiResolver>>, Body>;

pub struct CustomClient {
    client: Client,
    ratelimiters: [LeakyBucket; 4],
    upload: UploadData,
}

struct UploadData {
    secret: &'static str,
    url: &'static str,
}

impl From<&'static BotConfig> for UploadData {
    #[inline]
    fn from(config: &'static BotConfig) -> Self {
        Self {
            secret: &config.tokens.upload_secret,
            url: &config.upload_url,
        }
    }
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
            ratelimiter(1), // ShishaMezo
        ];

        Self {
            client,
            ratelimiters,
            upload: UploadData::from(BotConfig::get()),
        }
    }

    async fn ratelimit(&self, site: Site) {
        self.ratelimiters[site as usize].acquire_one().await
    }

    async fn make_get_request(&self, url: impl AsRef<str>, site: Site) -> Result<Bytes> {
        let url = url.as_ref();
        trace!("GET request to url {url}");

        let req = Request::builder()
            .uri(url)
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

        Self::error_for_status(response, url).await
    }

    async fn make_post_request<F: Serialize>(
        &self,
        url: impl AsRef<str>,
        site: Site,
        form: &F,
    ) -> Result<Bytes> {
        let url = url.as_ref();
        trace!("POST request to url {url}");

        let form_body = serde_urlencoded::to_string(form).context("failed to encode form")?;

        let req = Request::builder()
            .method(Method::POST)
            .uri(url)
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

        Self::error_for_status(response, url).await
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

    pub async fn upload_video(
        &self,
        title: &str,
        author: &str,
        path: impl AsRef<Path>,
    ) -> Result<UploadResponse> {
        let path = path.as_ref();

        let mut file = File::open(&path)
            .await
            .with_context(|| format!("failed to open file {path:?}"))?;

        let mut data = Vec::with_capacity(1_048_576);

        file.read_to_end(&mut data)
            .await
            .with_context(|| format!("failed to read file {path:?}"))?;

        trace!("uploading file of size {} bytes", data.len());

        let form = &[
            ("video", data.as_slice()),
            ("title", title.as_bytes()),
            ("author", author.as_bytes()),
            ("secret", self.upload.secret.as_bytes()),
        ];

        let bytes = self
            .make_post_request(self.upload.url, Site::ShishaMezo, form)
            .await?;

        serde_json::from_slice(&bytes).with_context(|| {
            let text = String::from_utf8_lossy(&bytes);

            format!("failed to deserialize upload response: {text}")
        })
    }
}

#[derive(Deserialize)]
pub struct UploadResponse {
    pub error: u16,
    pub text: String,
}
