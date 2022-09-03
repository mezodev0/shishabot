use eyre::{Context, Result};
use std::{
    borrow::Borrow,
    fmt::{Display, Formatter, Result as FmtResult},
    io::{Cursor, Result as IoResult},
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::fs;
use twilight_model::channel::Message;
use zip::ZipArchive;

use crate::{
    core::{BotConfig, Context as TwilightContext},
    util::{
        builder::MessageBuilder, constants::GENERAL_ISSUE, interaction::InteractionCommand,
        InteractionCommandExt,
    },
};

use super::SkinAdd;

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

pub async fn add(
    ctx: Arc<TwilightContext>,
    command: InteractionCommand,
    args: SkinAdd,
) -> Result<()> {
    let SkinAdd { skin } = args;
    if !matches!(skin.filename.split('.').last(), Some("osk")) {
        let content = "The attachment must be a .osk file!";
        command.error_callback(&ctx, content, true).await?;

        return Ok(());
    }

    {
        let content = "Downloading...";
        let builder = MessageBuilder::new().embed(content);
        command.callback(&ctx, builder, false).await?;
    }

    let bytes = match ctx.client().get_discord_attachment(&skin).await {
        Ok(bytes) => bytes,
        Err(err) => {
            command.error(&ctx, "Failed to download attachment").await?;

            return Err(err);
        }
    };

    {
        let content = "Zipping...";
        let builder = MessageBuilder::new().embed(content);
        command.update(&ctx, &builder).await?;
    }

    let cursor = Cursor::new(bytes);

    let mut archive = match ZipArchive::new(cursor) {
        Ok(archive) => archive,
        Err(err) => {
            warn!("failed to create zip archive: {err}");
            command.error(&ctx, GENERAL_ISSUE).await?;
            return Ok(());
        }
    };

    let config = BotConfig::get();
    let mut skin_file = config.paths.skins();

    {
        let content = "Generating skin name...";
        let builder = MessageBuilder::new().embed(content);
        command.update(&ctx, &builder).await?;
    }

    if let Some((filename, _extension)) = skin.filename.rsplit_once('.') {
        let mut file_count = FileCounter::new(filename);
        loop {
            skin_file.push(file_count.to_string());
            if !skin_file.exists() {
                break file_count.to_string();
            } else {
                skin_file.pop();
                file_count.inc();
            }
        }
    } else {
        warn!("failed to get filename");
        command.error(&ctx, GENERAL_ISSUE).await?;
        return Ok(());
    };

    {
        let content = "Extracting...";
        let builder = MessageBuilder::new().embed(content);
        command.update(&ctx, &builder).await?;
    }

    if let Err(err) = archive.extract(&skin_file) {
        warn!("failed to extract zip archive: {err}");
        command.error(&ctx, GENERAL_ISSUE).await?;

        return Ok(());
    }

    if !(PathBuf::from(format!("{:?}/skin.ini", &skin_file.as_path())).exists()
        || move_directory(&skin_file).await?)
    {
        let content = "There was an error getting the folder containing the skin elements! \
            Try re-exporting the skin!";
        command.error(&ctx, content).await?;
        fs::remove_dir_all(Path::new(&skin_file)).await?;

        return Ok(());
    }

    let mut skin_dir = fs::read_dir(config.paths.skins())
        .await
        .context("Failed to read skin dir")?;

    let mut counter = 0;
    let skin_file_name = skin_file.file_name().unwrap();
    while let Some(skin) = skin_dir
        .next_entry()
        .await
        .context("failed to get entry of skin dir")?
    {
        let file_name = skin.file_name();
        counter += 1;
        if file_name.to_os_string().eq(skin_file_name) {
            break;
        }
    }

    let content = format!("Added skin to list at index `{counter}`");
    let builder = MessageBuilder::new().embed(content);
    command.update(&ctx, &builder).await?;

    Ok(())
}

fn copy_all(from: PathBuf, to: PathBuf) -> IoResult<()> {
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

async fn move_directory(to: &PathBuf) -> IoResult<bool> {
    let mut skin_folder = fs::read_dir(&to).await?;

    let skin_folder_elements = if let Some(skin_folder_elements) = skin_folder.next_entry().await? {
        skin_folder_elements
    } else {
        return Ok(false);
    };

    let from = skin_folder_elements.path();
    let mut skin_ini = to.clone();
    skin_ini.push("skin.ini");
    if copy_all(from, to.clone()).is_ok() && PathBuf::from(skin_ini).exists() {
        fs::remove_dir_all(skin_folder_elements.path()).await?;

        Ok(true)
    } else {
        Ok(false)
    }
}
