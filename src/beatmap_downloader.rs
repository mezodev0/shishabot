pub async fn prepare_beatmap_file(map_id: u32) -> Result<String, MapDownloadError> {
    let mut map_path = "../Songs/".get().unwrap().map_path.clone();
    map_path.push(format!("{}.osu", map_id));

    if !map_path.exists() {
        let content = request_beatmap_file(map_id).await?;
        let mut file = File::create(&map_path).await?;
        file.write_all(&content).await?;
        info!("Downloaded {}.osu successfully", map_id);
    }

    let map_path = map_path
        .into_os_string()
        .into_string()
        .expect("map_path OsString is no valid String");

    Ok(map_path)
}

async fn request_beatmap_file(map_id: u32) -> Result<Bytes, MapDownloadError> {
    let url = format!("{}osu/{}", OSU_BASE, map_id);
    let mut content = reqwest::get(&url).await?.bytes().await?;

    if content.len() >= 6 && &content.slice(0..6)[..] != b"<html>" {
        return Ok(content);
    }

    // 500ms - 1000ms - 2000ms - 4000ms - 8000ms - 10000ms - 10000ms - ...
    let backoff = ExponentialBackoff::new(2).factor(250).max_delay(10_000);

    for (i, duration) in backoff.take(10).enumerate() {
        debug!("Retry attempt #{} | Backoff {:?}", i + 1, duration);
        sleep(duration).await;

        content = reqwest::get(&url).await?.bytes().await?;

        if content.len() >= 6 && &content.slice(0..6)[..] != b"<html>" {
            return Ok(content);
        }
    }

    (content.len() >= 6 && &content.slice(0..6)[..] != b"<html>")
        .then(|| content)
        .ok_or(MapDownloadError::Content(map_id))
}

#[derive(Debug, Clone)]
pub struct ExponentialBackoff {
    current: Duration,
    base: u32,
    factor: u32,
    max_delay: Option<Duration>,
}

impl ExponentialBackoff {
    pub fn new(base: u32) -> Self {
        ExponentialBackoff {
            current: Duration::from_millis(base as u64),
            base,
            factor: 1,
            max_delay: None,
        }
    }

    pub fn factor(mut self, factor: u32) -> Self {
        self.factor = factor;

        self
    }

    pub fn max_delay(mut self, max_delay: u64) -> Self {
        self.max_delay.replace(Duration::from_millis(max_delay));

        self
    }
}

impl Iterator for ExponentialBackoff {
    type Item = Duration;

    fn next(&mut self) -> Option<Duration> {
        let duration = self.current * self.factor;

        if let Some(max_delay) = self.max_delay.filter(|&max_delay| duration > max_delay) {
            return Some(max_delay);
        }

        self.current *= self.base;

        Some(duration)
    }
}