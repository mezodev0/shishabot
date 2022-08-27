use std::{
    fmt::{Display, Formatter},
    io::Cursor,
    path::Path,
};

use crate::checks::PERMISSIONS_CHECK;
use anyhow::Context;
use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};

use tokio::fs;
use zip::ZipArchive;

struct FileCounter {
    base: String,
    count: usize,
}

impl FileCounter {
    fn new(base: String) -> Self {
        Self { base, count: 0 }
    }

    fn inc(&mut self) {
        self.count += 1;
    }

    fn into_string(self) -> String {
        if self.count == 0 {
            self.base
        } else {
            self.base + "_" + &self.count.to_string()
        }
    }
}

impl Display for FileCounter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.count == 0 {
            f.write_str(&self.base)
        } else {
            write!(f, "{}_{}", self.base, self.count)
        }
    }
}

#[command]
#[checks(Permissions)]
#[only_in(guilds)]
#[description = "**Requires osu! skin Attachment"]
async fn addskin(ctx: &serenity::prelude::Context, msg: &Message) -> CommandResult {
    let attachment = match msg.attachments.last() {
        Some(a) if matches!(a.filename.split('.').last(), Some("osk")) => a,
        Some(_) | None => {
            msg.reply(&ctx, "The file you have sent is not a skin file!")
                .await?;
            return Ok(());
        }
    };

    let bytes = match attachment.download().await {
        Ok(bytes) => bytes,
        Err(err) => {
            warn!("skin download error: {err}");
            msg.reply(&ctx, "There was an issue downloading the file, blame mezo")
                .await?;
            return Ok(());
        }
    };

    let cursor = Cursor::new(bytes);

    let mut archive = match ZipArchive::new(cursor) {
        Ok(archive) => archive,
        Err(err) => {
            warn!("failed to create zip archive: {err}");
            msg.reply(&ctx, "Failed to create skin file, blame mezo")
                .await?;
            return Ok(());
        }
    };

    let skinname = if let Some((filename, _extension)) = attachment.filename.rsplit_once(".") {
        let mut file_count = FileCounter::new(filename.to_string());
        loop {
            if !Path::new(&format!("../Skins/{file_count}")).exists() {
                break file_count.into_string();
            } else {
                file_count.inc();
            }
        }
    } else {
        warn!("failed to get filename");
        msg.reply(&ctx, "Failed to get filename, blame mezo")
            .await?;
        return Ok(());
    };

    match archive.extract(format!("../Skins/{}", skinname)) {
        Ok(()) => (),
        Err(err) => {
            warn!("failed to extract zip archive: {err}");
            msg.reply(&ctx, "Failed to unzip skin file, blame mezo")
                .await?;
            return Ok(());
        }
    };

    let mut skins = fs::read_dir("../Skins/")
        .await
        .context("failed to read dir `../Skins/`")?;
    let mut counter = 0;

    while let Some(skin) = skins
        .next_entry()
        .await
        .context("failed to get entry of `../Skins/`")?
    {
        let file_name = skin.file_name();
        counter += 1;

        if file_name.to_string_lossy() == skinname {
            break;
        }
    }

    msg.reply(&ctx, format!("Added skin to list at index `{counter}`"))
        .await?;

    Ok(())
}
