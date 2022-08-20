use std::path::Path;

use anyhow::{Context, Result};
use reqwest::{
    multipart::{self, Part},
    Client, Response,
};
use serde::Deserialize;
use tokio::{fs::File, io::AsyncReadExt};

#[derive(Clone)]
pub struct CustomUploadApi {
    pub url: String,
    pub client: Client,
    pub secret_key: String,
}

#[derive(Deserialize)]
pub struct UploadResponse {
    pub link: String,
}

impl CustomUploadApi {
    pub async fn new(url: String, secret_key: String) -> Result<Self> {
        Ok(Self {
            url,
            client: reqwest::Client::new(),
            secret_key,
        })
    }
    pub async fn upload_video(&self, title: String, filepath: &str) -> Result<UploadResponse> {
        let resp = self.api_request(title, filepath).await?;
        let json = resp.json::<UploadResponse>().await?;

        Ok(json)
    }

    pub async fn api_request(&self, data: String, files: &str) -> Result<Response> {
        let file = read_file(&files)
            .await
            .with_context(|| format!("failed to load file for path `{files}`"))?;

        let form = multipart::Form::new()
            .part("video", file)
            .text("title", data)
            .text("secret", self.secret_key.clone());

        let resp = self
            .client
            .post(self.url.clone())
            .multipart(form)
            .send()
            .await?;

        Ok(resp)
    }
}

pub async fn read_file<T: AsRef<Path>>(path: T) -> Result<Part> {
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
