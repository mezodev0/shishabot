use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    io::{Cursor, Result as IoResult},
    path::{Path, PathBuf},
};

use anyhow::Context as _;
use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};
use tokio::fs;
use zip::ZipArchive;

use crate::checks::BOTOWNER_CHECK;

struct FileCounter<'s> {
    base: &'s str,
    count: usize,
}

impl<'s> FileCounter<'s> {
    fn new(base: &'s str) -> Self {
        Self { base, count: 0 }
    }

    fn inc(&mut self) {
        self.count += 1;
    }
}

impl Display for FileCounter<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if self.count == 0 {
            f.write_str(self.base)
        } else {
            write!(f, "{}_{}", self.base, self.count)
        }
    }
}

#[command]
#[checks(BotOwner)]
#[description = "**Requires osu! skin attachment**\nAllows you to upload custom skins"]
async fn addskin(ctx: &Context, msg: &Message) -> CommandResult {
    let attachment = match msg.attachments.last() {
        Some(a) if matches!(a.filename.split('.').last(), Some("osk")) => a,
        Some(_) | None => {
            msg.reply(&ctx, "The file you have sent is not a skin file!")
                .await?;
            return Ok(());
        }
    };

    let mut resp = msg.reply(&ctx, "Downloading skin...").await?;

    let bytes = match attachment.download().await {
        Ok(bytes) => bytes,
        Err(err) => {
            warn!("skin download error: {err}");
            msg.reply(&ctx, "There was an issue downloading the file, blame mezo")
                .await?;
            return Ok(());
        }
    };

    resp.edit(&ctx, |m| m.content("Creating archive..."))
        .await?;

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

    resp.edit(&ctx, |m| m.content("Generating skin name..."))
        .await?;

    let skinname = if let Some((filename, _extension)) = attachment.filename.rsplit_once('.') {
        let mut file_count = FileCounter::new(filename);
        loop {
            if !Path::new(&format!("../Skins/{file_count}")).exists() {
                break file_count.to_string();
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

    resp.edit(&ctx, |m| m.content("Extracting archive..."))
        .await?;

    if let Err(err) = archive.extract(format!("../Skins/{skinname}")) {
        warn!("failed to extract zip archive: {err}");
        msg.reply(&ctx, "Failed to unzip skin file, blame mezo")
            .await?;

        return Ok(());
    }

    resp.edit(&ctx, |m| m.content("Checking for `skin.ini`..."))
        .await?;

    if !(PathBuf::from(format!("../Skins/{skinname}/skin.ini")).exists()
        || move_directory(&skinname).await?)
    {
        let content = "There was an error getting the folder containing the skin elements! \
            Try re-exporting the skin!";
        resp.edit(ctx, |m| m.content(content)).await?;
        fs::remove_dir_all(Path::new(&format!("../Skins/{skinname}"))).await?;

        return Ok(());
    } else {
        resp.edit(&ctx, |m| m.content("Found `skin.ini`!")).await?;
    }

    let mut skins = fs::read_dir("../Skins/")
        .await
        .context("failed to read dir `../Skins/`")?;

    let mut counter = 0;

    while skins
        .next_entry()
        .await
        .context("failed to get entry of `../Skins/`")?
        .is_some()
    {
        counter += 1;
    }

    let content = format!("Added skin to list at index `{counter}`");
    resp.edit(&ctx, |m| m.content(content)).await?;

    Ok(())
}

pub fn copy_all(from: PathBuf, to: PathBuf) -> IoResult<()> {
    let output_root = to;
    let input_root = from.components().count();

    let mut stack = vec![from];

    while let Some(working_path) = stack.pop() {
        // Generate a relative path
        let src: PathBuf = working_path.components().skip(input_root).collect();

        // Create a destination if missing
        let dest = if src.components().count() == 0 {
            output_root.clone()
        } else {
            output_root.join(&src)
        };

        if std::fs::metadata(&dest).is_err() {
            std::fs::create_dir_all(&dest)?;
        }

        for entry in std::fs::read_dir(working_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                stack.push(path);
            } else if let Some(filename) = path.file_name() {
                let dest_path = dest.join(filename);
                std::fs::copy(&path, &dest_path)?;
            }
        }
    }

    Ok(())
}

async fn move_directory(skinname: &str) -> IoResult<bool> {
    let to = PathBuf::from(format!("../Skins/{skinname}"));
    let mut skin_folder = fs::read_dir(&to).await?;

    let skin_folder_elements = if let Some(skin_folder_elements) = skin_folder.next_entry().await? {
        skin_folder_elements
    } else {
        return Ok(false);
    };

    let from = skin_folder_elements.path();

    if copy_all(from, to).is_ok() && PathBuf::from(format!("../Skins/{skinname}/skin.ini")).exists()
    {
        fs::remove_dir_all(skin_folder_elements.path()).await?;

        Ok(true)
    } else {
        Ok(false)
    }
}
