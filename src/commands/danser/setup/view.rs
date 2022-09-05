use std::{fmt::Write, sync::Arc};

use eyre::Result;

use crate::{
    core::Context,
    util::{builder::MessageBuilder, interaction::InteractionCommand, InteractionCommandExt},
};

pub async fn view(ctx: Arc<Context>, command: InteractionCommand) -> Result<()> {
    let guild_id = command.guild_id.unwrap();

    let input_channels = ctx
        .guild_settings(guild_id, |server| {
            let mut iter = server.input_channels.iter();

            iter.next().map(|channel| {
                let mut text = format!("<#{channel}>");

                for channel in iter {
                    let _ = write!(text, ", <#{channel}>");
                }

                text
            })
        })
        .flatten()
        .unwrap_or_else(|| "None".to_owned());

    let output_channel = ctx
        .guild_settings(guild_id, |s| s.output_channel)
        .flatten()
        .map_or_else(|| "None".to_owned(), |channel| format!("<#{channel}>"));

    let content = format!("Input channels: {input_channels}\nOutput channel: {output_channel}");
    let builder = MessageBuilder::new().embed(content);
    command.callback(&ctx, builder, false).await?;

    Ok(())
}
