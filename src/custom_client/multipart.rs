use std::{ffi::OsStr, fmt::Display, io::Write, path::Path};

use eyre::{Context as _, Result};
use rand::{distributions::Alphanumeric, Rng};
use tokio::{fs::File, io};

const BOUNDARY_LEN: usize = 16;

pub struct Multipart {
    bytes: Vec<u8>,
    boundary: String,
}

impl Multipart {
    pub fn new() -> Self {
        let boundary = rand::thread_rng()
            .sample_iter(Alphanumeric)
            .take(BOUNDARY_LEN)
            .map(|c| c as char)
            .collect();

        Self {
            bytes: Vec::with_capacity(1_048_576),
            boundary,
        }
    }

    pub fn push_text<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Display,
        V: Display,
    {
        self.write_field_headers(key, None, None);
        let _ = write!(self.bytes, "{value}");

        self
    }

    pub async fn push_file<K, P>(mut self, key: K, path: P) -> Result<Self>
    where
        K: Display,
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let extension = path.extension();

        ensure!(
            matches!(extension.and_then(OsStr::to_str), Some("mp4")),
            "unexpected file extension {extension:?} while creating multipart, expected mp4",
        );

        let filename = path.file_name().and_then(OsStr::to_str);

        let mut file = File::open(path)
            .await
            .with_context(|| format!("failed to open file at {path:?}"))?;

        self.write_field_headers(key, filename, Some("video/mp4"));

        io::copy(&mut file, &mut self.bytes)
            .await
            .with_context(|| format!("failed to copy bytes from {path:?}"))?;

        Ok(self)
    }

    pub fn finish(mut self) -> Vec<u8> {
        if !self.is_empty() {
            self.bytes.extend_from_slice(b"\r\n");
        }

        let _ = write!(self.bytes, "--{}--\r\n", self.boundary);

        self.bytes
    }

    pub fn boundary(&self) -> &str {
        &self.boundary
    }

    fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    fn write_field_headers(
        &mut self,
        name: impl Display,
        filename: Option<&str>,
        content_type: Option<&str>,
    ) {
        if !self.is_empty() {
            self.bytes.extend_from_slice(b"\r\n");
        }

        let _ = write!(self.bytes, "--{}\r\n", self.boundary);

        let _ = write!(
            self.bytes,
            "Content-Disposition: form-data; name=\"{name}\""
        );

        if let Some(filename) = filename {
            let _ = write!(self.bytes, "; filename=\"{filename}\"");
        }

        if let Some(content_type) = content_type {
            let _ = write!(self.bytes, "\r\nContent-Type: {content_type}");
        }

        self.bytes.extend_from_slice(b"\r\n\r\n");
    }
}

#[cfg(test)]
mod tests {
    use super::Multipart;

    #[test]
    fn empty() {
        let form = Multipart::new();

        let expect = format!("--{}--\r\n", form.boundary());

        let form = String::from_utf8(form.finish()).unwrap();

        assert_eq!(form, expect);
    }

    #[test]
    fn texts() {
        let form = Multipart::new()
            .push_text("key1", "value1")
            .push_text("key2", "value2");

        let boundary = form.boundary();

        let expect = format!(
            "--{boundary}\r\n\
            Content-Disposition: form-data; name=\"key1\"\r\n\r\n\
            value1\r\n\
            --{boundary}\r\n\
            Content-Disposition: form-data; name=\"key2\"\r\n\r\n\
            value2\r\n--{boundary}--\r\n"
        );

        let form = String::from_utf8(form.finish()).unwrap();

        assert_eq!(form, expect);
    }

    #[tokio::test]
    #[should_panic]
    async fn non_mp4() {
        let _ = Multipart::new()
            .push_file("cargo", "./Cargo.toml")
            .await
            .unwrap();
    }
}
