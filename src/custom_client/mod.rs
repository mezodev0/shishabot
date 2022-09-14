use std::{hash::Hash, path::Path};

use crate::core::{replay_queue::ReplaySlim, BotConfig};
use bytes::Bytes;
use eyre::{Context as _, Result};
use http::{header::CONTENT_LENGTH, Response};
use hyper::{
    client::{connect::dns::GaiResolver, Client as HyperClient, HttpConnector},
    header::{CONTENT_TYPE, USER_AGENT},
    Body, Method, Request,
};
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use leaky_bucket_lite::LeakyBucket;
use serde::Deserialize;
use tokio::time::Duration;
use twilight_model::{
    channel::Attachment,
    id::{marker::UserMarker, Id},
};

use self::multipart::Multipart;

mod multipart;

static MY_USER_AGENT: &str = env!("CARGO_PKG_NAME");

#[derive(Copy, Clone, Eq, Hash, PartialEq)]
#[repr(u8)]
enum Site {
    DiscordAttachment,
    DownloadChimu,
    DownloadKitsu,
    OsuApi,
    ShishaMezo,
}

type Client = HyperClient<HttpsConnector<HttpConnector<GaiResolver>>, Body>;

pub struct CustomClient {
    client: Client,
    ratelimiters: [LeakyBucket; 5],
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
            ratelimiter(1), // OsuApi
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

    async fn make_osu_get_request(&self, url: impl AsRef<str>, site: Site) -> Result<Bytes> {
        let url = url.as_ref();
        trace!("GET request to url {url}");

        let req = Request::builder()
            .uri(url)
            .method(Method::GET)
            .header(USER_AGENT, MY_USER_AGENT)
            .header(
                hyper::header::COOKIE,
                format!(
                    "osu_session={};",
                    BotConfig::get().tokens.osu_session_cookie
                ),
            )
            .header(hyper::header::REFERER, url)
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

    async fn make_post_request(
        &self,
        url: impl AsRef<str>,
        site: Site,
        form: Multipart,
    ) -> Result<Bytes> {
        let url = url.as_ref();
        trace!("POST request to url {url}");

        let content_type = format!("multipart/form-data; boundary={}", form.boundary());
        let form = form.finish();

        let req = Request::builder()
            .method(Method::POST)
            .uri(url)
            .header(USER_AGENT, MY_USER_AGENT)
            .header(CONTENT_TYPE, content_type)
            .header(CONTENT_LENGTH, form.len())
            .body(Body::from(form))
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

    pub async fn get_osu_replay(&self, score_id: u64) -> Result<ReplaySlim> {
        let bytes = self
            .make_osu_get_request(
                format!("https://osu.ppy.sh/scores/osu/{score_id}/download"),
                Site::OsuApi,
            )
            .await
            .context("failed to download osu replay")?;

        let mut path = BotConfig::get().paths.downloads();
        path.push(format!("{score_id}.osr"));

        std::fs::write(path.clone(), &bytes).context("failed to create replay file")?;

        let replay =
            osu_db::Replay::from_file(path.as_path()).context("failed to get replay from file")?;
        let replay_slim = ReplaySlim::from(replay);

        Ok(replay_slim)
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
        author: Id<UserMarker>,
        path: impl AsRef<Path>,
    ) -> Result<UploadResponse> {
        let form = Multipart::new()
            .push_file("video", path)
            .await
            .context("failed to create multipart form")?
            .push_text("title", title)
            .push_text("author", author)
            .push_text("secret", self.upload.secret);

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

#[derive(Deserialize)]
pub struct OsuReplayResponse {
    pub content: String,
    pub encoding: String,
}
