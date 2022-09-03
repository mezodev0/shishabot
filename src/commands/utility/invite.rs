use std::sync::Arc;

use command_macros::{command, SlashCommand};
use eyre::Result;
use twilight_interactions::command::CreateCommand;

use crate::{
    util::{
        builder::{EmbedBuilder, FooterBuilder, MessageBuilder},
        constants::INVITE_LINK,
        interaction::InteractionCommand,
        InteractionCommandExt,
    },
    Context, DEFAULT_PREFIX,
};

#[derive(CreateCommand, SlashCommand)]
#[command(name = "invite")]
#[flags(SKIP_DEFER)]
/// Invite me to your server
pub struct Invite;

pub async fn slash_invite(ctx: Arc<Context>, mut command: InteractionCommand) -> Result<()> {
    let footer = format!("The initial prefix will be {DEFAULT_PREFIX}");

    let embed = EmbedBuilder::new()
        .description(INVITE_LINK)
        .footer(FooterBuilder::new(footer))
        .title("Invite me to your server!");

    let builder = MessageBuilder::new().embed(embed);
    command.callback(&ctx, builder, false).await?;

    Ok(())
}
