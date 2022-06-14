use std::path::Path;

use anyhow::{Context, Error};
use base64::encode;
use reqwest::{
    self,
    header::{self, HeaderMap},
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

impl StreamableApi {
    pub async fn new(username: String, password: String) -> reqwest::Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(
                &format!("Basic {}", encode(format!("{}:{}", username, password))).as_str(),
            )
            .unwrap(),
        );
        let client = Client::builder().default_headers(headers).build()?;

        Ok(Self { client })
    }
    pub async fn upload_video(
        &self,
        filename: String,
        title: Option<String>,
        filepath: String,
    ) -> Result<Response, reqwest::Error> {
        let mut video_title = filename;
        if title != None {
            video_title = title.unwrap();
        }

        let resp = self
            .api_request(
                "https://api.streamable.com/upload".to_string(),
                video_title,
                filepath,
            )
            .await?;
        Ok(resp)
    }

    pub async fn check_status(&self, shortcode: String) -> Result<StatusResponse, Error> {
        let url = format!("https://api.streamable.com/videos/{}", shortcode);
        let resp = self.client.get(url).send().await?.text().await?;
        let custom_resp: StatusResponse = serde_json::from_str(&resp)?;
        Ok(custom_resp)
    }

    pub async fn api_request(
        &self,
        url: String,
        data: String,
        files: String,
    ) -> Result<Response, reqwest::Error> {
        // let mut data_params = HashMap::new();
        // data_params.insert("title", data);

        let file = readfile(&files)
            .await
            .with_context(|| format!("failed to load file for path `{}`", files))
            .unwrap();

        let form = multipart::Form::new()
            .part("file", file)
            .text("title", data);
        let resp = self.client.post(url).multipart(form).send().await?;
        Ok(resp)
    }
}

pub async fn readfile<T: AsRef<Path>>(path: T) -> Result<Part, Error> {
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
