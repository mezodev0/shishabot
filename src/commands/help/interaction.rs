use std::sync::Arc;

use command_macros::SlashCommand;
use eyre::Result;
use twilight_interactions::command::CreateCommand;
use twilight_model::channel::embed::EmbedField;

use crate::{
    core::Context,
    util::{
        builder::{EmbedBuilder, MessageBuilder},
        constants::{INVITE_LINK, SHISHABOT_DISCORD, SHISHABOT_GITHUB, SHISHABOT_WEBSITE},
        datetime::how_long_ago_dynamic,
        interaction::InteractionCommand,
        numbers::with_comma_int,
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
    let id = ctx
        .cache
        .current_user(|user| user.id)
        .expect("missing CurrentUser in cache");

    let mention = format!("<@{id}>");

    let description =
        format!("{mention} is a discord bot written by [mezo](https://osu.ppy.sh/users/13313647) which allows you render osu! replays and upload them");

    let view_replays = EmbedField {
        inline: false,
        name: "Do you want to view your replays?".to_owned(),
        value: format!(
            "Head over to [shisha.mezo.xyz]({SHISHABOT_WEBSITE}) and log in using discord"
        ),
    };

    let join_server = EmbedField {
        inline: false,
        name: "Got a question, suggestion, bug, or are interested in the development?".to_owned(),
        value: format!("Feel free to join the [discord server]({SHISHABOT_DISCORD})"),
    };

    let invite = EmbedField {
        inline: false,
        name: "Want to invite the bot to your server?".to_owned(),
        value: format!("Try using this [**invite link**]({INVITE_LINK})"),
    };

    let servers = EmbedField {
        inline: true,
        name: "Servers".to_owned(),
        value: with_comma_int(ctx.cache.stats().guilds()).to_string(),
    };

    let boot_time = ctx.stats.start_time;

    let boot_up = EmbedField {
        inline: true,
        name: "Boot-up".to_owned(),
        value: how_long_ago_dynamic(&boot_time).to_string(),
    };

    let github = EmbedField {
        inline: false,
        name: "Interested in the code?".to_owned(),
        value: format!("The source code can be found over at [Github]({SHISHABOT_GITHUB})"),
    };

    let fields = vec![view_replays, join_server, invite, servers, boot_up, github];

    let embed = EmbedBuilder::new().description(description).fields(fields);

    let menus = generate_menus(&[]);

    let builder = MessageBuilder::new().embed(embed).components(menus);

    command.callback(&ctx, builder, true).await?;

    Ok(())
}
