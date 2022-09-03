use std::sync::Arc;

use command_macros::SlashCommand;
use eyre::Result;
use twilight_interactions::command::CreateCommand;

use crate::{
    core::Context,
    util::{
        builder::{EmbedBuilder, MessageBuilder},
        interaction::InteractionCommand,
        InteractionCommandExt,
    },
};

use super::generate_menus;

#[derive(CreateCommand, SlashCommand)]
#[flags(SKIP_DEFER)]
#[command(name = "help")]
/// Display general help or help for specific commands
pub struct Help;

pub async fn slash_help(ctx: Arc<Context>, command: InteractionCommand) -> Result<()> {
    let description = "bla bla mezo edit this";

    let embed = EmbedBuilder::new().description(description);

    let menus = generate_menus(&[]);

    let builder = MessageBuilder::new().embed(embed).components(menus);

    command.callback(&ctx, builder, true).await?;

    Ok(())
}
