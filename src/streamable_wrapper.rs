use std::path::Path;

use anyhow::{Context, Result};
use base64::encode;
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION},
    multipart::{self, Part},
    Client, Response,
};
use serde::Deserialize;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

pub struct StreamableApi {
    pub client: Client,
}

#[derive(Deserialize)]
pub struct StatusResponse {
    pub status: i8,
}

#[derive(Deserialize)]
pub struct UploadResponse {
    pub shortcode: String,
    pub status: i8,
}

impl StreamableApi {
    pub async fn new(username: String, password: String) -> Result<Self> {
        let mut headers = HeaderMap::new();
        let value = format!("Basic {}", encode(format!("{username}:{password}")));
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&value)?);
        let client = Client::builder().default_headers(headers).build()?;

        Ok(Self { client })
    }

    pub async fn upload_video(&self, title: String, filepath: &str) -> Result<UploadResponse> {
        let url = "https://api.streamable.com/upload";
        let resp = self.api_request(url, title, filepath).await?;
        let json = resp.json::<UploadResponse>().await?;

        Ok(json)
    }

    pub async fn check_status_code(&self, shortcode: &str) -> Result<i8> {
        let url = format!("https://api.streamable.com/videos/{shortcode}");
        let resp = self.client.get(url).send().await?.bytes().await?;
        let custom_resp: StatusResponse = serde_json::from_slice(&resp)?;

        Ok(custom_resp.status)
    }

    pub async fn api_request(&self, url: &str, data: String, files: &str) -> Result<Response> {
        let file = readfile(&files)
            .await
            .with_context(|| format!("failed to load file for path `{files}`"))?;

        let form = multipart::Form::new()
            .part("file", file)
            .text("title", data);

        let resp = self.client.post(url).multipart(form).send().await?;

        Ok(resp)
    }
}

pub async fn readfile<T: AsRef<Path>>(path: T) -> Result<Part> {
    let path = path.as_ref();

    let file_name = path
        .file_name()
        .map(|filename| filename.to_string_lossy().into_owned());

    let ext = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
    let mime = mime_guess::from_ext(ext).first_or_octet_stream();

    let mut file = File::open(path)
        .await
        .with_context(|| format!("failed to open file `{}`", path.display()))?;

    let mut bytes = Vec::new();

    file.read_to_end(&mut bytes)
        .await
        .with_context(|| format!("failed to read file `{}`", path.display()))?;

    let field = Part::bytes(bytes).mime_str(mime.essence_str())?;

    let part = if let Some(file_name) = file_name {
        field.file_name(file_name)
    } else {
        field
    };

    Ok(part)
}
