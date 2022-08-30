use std::{fmt::Display, hash::Hash};

use bytes::Bytes;
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

use crate::{
    core::BotConfig,
    util::{constants::OSU_BASE, ExponentialBackoff},
};

pub use self::error::*;

mod deserialize;
mod error;

type ClientResult<T> = Result<T, CustomClientError>;

static MY_USER_AGENT: &str = env!("CARGO_PKG_NAME");

const APPLICATION_JSON: &str = "application/json";
const APPLICATION_URLENCODED: &str = "application/x-www-form-urlencoded";

#[derive(Copy, Clone, Eq, Hash, PartialEq)]
#[repr(u8)]
enum Site {
    DiscordAttachment,
    Huismetbenen,
    Osekai,
    OsuAvatar,
    OsuBadge,
    OsuMapFile,
    OsuMapsetCover,
    OsuStats,
    OsuTracker,
    Respektive,
}

type Client = HyperClient<HttpsConnector<HttpConnector<GaiResolver>>, Body>;

pub struct CustomClient {
    client: Client,
    ratelimiters: [LeakyBucket; 10],
}

impl CustomClient {
    pub async fn new() -> ClientResult<Self> {
        let connector = HttpsConnectorBuilder::new()
            .with_webpki_roots()
            .https_or_http()
            .enable_http1()
            .build();

        let config = BotConfig::get();
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
            ratelimiter(2),  // DiscordAttachment
            ratelimiter(2),  // Huismetbenen
            ratelimiter(2),  // Osekai
            ratelimiter(10), // OsuAvatar
            ratelimiter(10), // OsuBadge
            ratelimiter(5),  // OsuMapFile
            ratelimiter(10), // OsuMapsetCover
            ratelimiter(2),  // OsuStats
            ratelimiter(2),  // OsuTracker
            ratelimiter(1),  // Respektive
        ];

        Ok(Self {
            client,
            ratelimiters,
        })
    }

    async fn ratelimit(&self, site: Site) {
        self.ratelimiters[site as usize].acquire_one().await
    }

    async fn make_get_request(&self, url: impl AsRef<str>, site: Site) -> ClientResult<Bytes> {
        trace!("GET request of url {}", url.as_ref());

        let req = Request::builder()
            .uri(url.as_ref())
            .method(Method::GET)
            .header(USER_AGENT, MY_USER_AGENT)
            .body(Body::empty())?;

        self.ratelimit(site).await;
        let response = self.client.request(req).await?;

        Self::error_for_status(response, url.as_ref()).await
    }

    #[cfg(debug_assertions)]
    async fn make_twitch_get_request<I, U, V>(
        &self,
        _: impl AsRef<str>,
        _: I,
    ) -> ClientResult<Bytes>
    where
        I: IntoIterator<Item = (U, V)>,
        U: Display,
        V: Display,
    {
        Err(CustomClientError::NoTwitchOnDebug)
    }

    async fn make_post_request<F: Serialize>(
        &self,
        url: impl AsRef<str>,
        site: Site,
        form: &F,
    ) -> ClientResult<Bytes> {
        trace!("POST request of url {}", url.as_ref());

        let form_body = serde_urlencoded::to_string(form)?;

        let req = Request::builder()
            .method(Method::POST)
            .uri(url.as_ref())
            .header(USER_AGENT, MY_USER_AGENT)
            .header(CONTENT_TYPE, APPLICATION_URLENCODED)
            .body(Body::from(form_body))?;

        self.ratelimit(site).await;
        let response = self.client.request(req).await?;

        Self::error_for_status(response, url.as_ref()).await
    }

    async fn error_for_status(
        response: Response<Body>,
        url: impl Into<String>,
    ) -> ClientResult<Bytes> {
        if response.status().is_client_error() || response.status().is_server_error() {
            Err(CustomClientError::Status {
                status: response.status(),
                url: url.into(),
            })
        } else {
            let bytes = hyper::body::to_bytes(response.into_body()).await?;

            Ok(bytes)
        }
    }

    pub async fn get_discord_attachment(&self, attachment: &Attachment) -> ClientResult<Bytes> {
        self.make_get_request(&attachment.url, Site::DiscordAttachment)
            .await
    }

    pub async fn get_map_file(&self, map_id: u32) -> ClientResult<Bytes> {
        let url = format!("{OSU_BASE}osu/{map_id}");
        let backoff = ExponentialBackoff::new(2).factor(500).max_delay(10_000);
        const ATTEMPTS: usize = 10;

        for (duration, i) in backoff.take(ATTEMPTS).zip(1..) {
            let result = self.make_get_request(&url, Site::OsuMapFile).await;

            if matches!(&result, Err(CustomClientError::Status { status, ..}) if *status == StatusCode::TOO_MANY_REQUESTS)
                || matches!(&result, Ok(bytes) if bytes.starts_with(b"<html>"))
            {
                debug!("Request beatmap retry attempt #{i} | Backoff {duration:?}");
                sleep(duration).await;
            } else {
                return result;
            }
        }

        Err(CustomClientError::MapFileRetryLimit(map_id))
    }
}
