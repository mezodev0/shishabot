use std::{
    fmt::{Display, Formatter},
    io::Cursor,
    path::{Path, PathBuf},
};

use crate::checks::BOTOWNER_CHECK;
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
#[checks(BotOwner)]
#[description = "**Requires osu! skin attachment**\nAllows you to upload custom skins"]
async fn addskin(ctx: &serenity::prelude::Context, msg: &Message) -> CommandResult {
    
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

    resp.edit(&ctx, |m| m.content("Creating archive...")).await?;

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

    resp.edit(&ctx, |m| m.content("Generating skin name...")).await?;

    let skinname = if let Some((filename, _extension)) = attachment.filename.rsplit_once('.') {
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

    resp.edit(&ctx, |m| m.content("Extracting archive...")).await?;

    match archive.extract(format!("../Skins/{}", skinname)) {
        Ok(()) => (),
        Err(err) => {
            warn!("failed to extract zip archive: {err}");
            msg.reply(&ctx, "Failed to unzip skin file, blame mezo")
                .await?;
            return Ok(());
        }
    };
    
    resp.edit(&ctx, |m| m.content("Checking for `skin.ini`...")).await?;

    if Path::new(&format!("../Skins/{}/skin.ini", &skinname)).exists() {
        resp.edit(&ctx, |m| m.content("Found `skin.ini`!")).await?;
    } else {
        let mut skin_folder = fs::read_dir(format!("../Skins/{}", &skinname)).await?;
        let skin_folder_elements = if let Some(skin_folder_elements) = skin_folder.next_entry().await? {
            skin_folder_elements
        } else {
            resp.edit(&ctx, |m| m.content("There was an error getting the folder containing the skin elements! Try re-exporting the skin!")).await?;
            fs::remove_dir_all(Path::new(&format!("../Skins/{}", &skinname))).await?;
            return Ok(());
        };
        if copy_all(skin_folder_elements.path(), Path::new(&format!("../Skins/{}", &skinname))).is_err() {
            resp.edit(&ctx, |m| m.content("There was an error getting the folder containing the skin elements! Try re-exporting the skin!")).await?;
            fs::remove_dir_all(Path::new(&format!("../Skins/{}", &skinname))).await?;
            return Ok(());
        } else {
            if Path::new(&format!("../Skins/{}/skin.ini", &skinname)).exists() {
                resp.edit(&ctx, |m| m.content("Found `skin.ini`!")).await?;
                fs::remove_dir_all(skin_folder_elements.path()).await?;
            } else {
                resp.edit(&ctx, |m| m.content("There was an error getting the folder containing the skin elements! Try re-exporting the skin!")).await?;
                fs::remove_dir_all(Path::new(&format!("../Skins/{}", &skinname))).await?;
                return Ok(());
            }
        }
    }

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

    resp.edit(&ctx, |m| m.content(format!("Added skin to list at index `{counter}`"))).await?;

    Ok(())
}

pub fn copy_all<U: AsRef<Path>, V: AsRef<Path>>(from: U, to: V) -> Result<(), std::io::Error> {
    let mut stack = Vec::new();
    stack.push(PathBuf::from(from.as_ref()));

    let output_root = PathBuf::from(to.as_ref());
    let input_root = PathBuf::from(from.as_ref()).components().count();

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
            } else {
                match path.file_name() {
                    Some(filename) => {
                        let dest_path = dest.join(filename);
                        std::fs::copy(&path, &dest_path)?;
                    }
                    None => {
                        unreachable!()
                    }
                }
            }
        }
    }

    Ok(())
}