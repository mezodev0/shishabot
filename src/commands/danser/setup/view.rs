use eyre::Result;
use std::{collections::HashSet, fmt::Write, sync::Arc};

use crate::{
    core::Context,
    util::{
        builder::MessageBuilder, interaction::InteractionCommand, Authored, InteractionCommandExt,
    },
};

pub async fn view(ctx: Arc<Context>, command: InteractionCommand) -> Result<()> {
    let input_channels = if let Some(input_channels) =
        ctx.guild_settings(command.guild_id.unwrap(), |s| s.input_channels.clone())
    {
        input_channels
    } else {
        HashSet::new()
    };

    let output_channel = if let Some(output_channel) =
        ctx.guild_settings(command.guild_id.unwrap(), |s| s.output_channel)
    {
        output_channel
    } else {
        None
    };

    let mut iter = input_channels.iter();

    let input_channel_text = if let Some(channel) = iter.next() {
        let mut text = format!("<#{}>", channel.to_string());

        for channel in iter {
            let _ = write!(text, ", <#{channel}>");
        }

        text
    } else {
        "None".to_owned()
    };

    let output_channel_text = if let Some(channel) = output_channel {
        format!("<#{channel}>")
    } else {
        "None".to_owned()
    };

    let content =
        format!("Input channels: {input_channel_text}\nOutput channel: {output_channel_text}");
    let builder = MessageBuilder::new().embed(content);

    command.callback(&ctx, builder, false).await;
    Ok(())
}
