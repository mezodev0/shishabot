use std::{fs::File, io::ErrorKind, sync::Arc};

use eyre::{Report, Result};
use twilight_interactions::command::ResolvedUser;

use crate::{
    core::{BotConfig, Context},
    util::{builder::MessageBuilder, interaction::InteractionCommand, InteractionCommandExt},
};

use super::{create_settings_embed, SettingsView};

pub async fn view(
    ctx: Arc<Context>,
    command: InteractionCommand,
    args: SettingsView,
) -> Result<()> {
    let user = args.user.resolved;

    let mut user_path = BotConfig::get().paths.danser().to_owned();
    user_path.push(format!("settings/{}.json", user.id));

    let settings = match File::open(&user_path) {
        Ok(file) => match serde_json::from_reader(file) {
            Ok(settings) => settings,
            Err(err) => {
                let content = "Failed to read settings file";
                let _ = command.error_callback(&ctx, content, false).await;

                let err = Report::from(err)
                    .wrap_err(format!("Failed to deserialize file at {user_path:?}"));

                return Err(err);
            }
        },
        Err(err) if err.kind() == ErrorKind::NotFound => {
            let content = format!("User <@{}> has no specified settings yet", user.id);
            let builder = MessageBuilder::new().embed(content);
            command.callback(&ctx, builder, false).await?;

            return Ok(());
        }
        Err(err) => {
            let content = "Failed to open settings file";
            let _ = command.error_callback(&ctx, content, false).await;
            let err = Report::from(err).wrap_err("failed to open settings file");

            return Err(err);
        }
    };

    let embed = create_settings_embed(&user, &settings);
    let builder = MessageBuilder::new().embed(embed);
    command.callback(&ctx, builder, false).await?;

    Ok(())
}
