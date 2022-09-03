use std::sync::Arc;

use command_macros::SlashCommand;
use eyre::Result;
use twilight_interactions::command::CreateCommand;

use crate::{
    util::{
        builder::{EmbedBuilder, MessageBuilder},
        constants::INVITE_LINK,
        interaction::InteractionCommand,
        InteractionCommandExt,
    },
    Context,
};

#[derive(CreateCommand, SlashCommand)]
#[command(name = "invite")]
#[flags(SKIP_DEFER)]
/// Invite me to your server
pub struct Invite;

pub async fn slash_invite(ctx: Arc<Context>, command: InteractionCommand) -> Result<()> {
    let embed = EmbedBuilder::new()
        .description(INVITE_LINK)
        .title("Invite me to your server!");

    let builder = MessageBuilder::new().embed(embed);
    command.callback(&ctx, builder, false).await?;

    Ok(())
}
