use std::{collections::BTreeMap, sync::Arc};

use command_macros::SlashCommand;
use eyre::Result;
use twilight_interactions::command::{
    ApplicationCommandData, AutocompleteValue, CommandModel, CreateCommand,
};
use twilight_model::{application::command::CommandOptionChoice, channel::embed::EmbedField};

use crate::{
    core::{
        commands::slash::{SlashCommand, SlashCommands},
        Context,
    },
    util::{
        builder::{EmbedBuilder, FooterBuilder, MessageBuilder},
        constants::{INVITE_LINK, SHISHABOT_GITHUB},
        interaction::InteractionCommand,
        levenshtein_distance,
        numbers::with_comma_int,
        CowUtils, InteractionCommandExt,
    },
};

use super::{failed_message_content, option_fields, parse_select_menu, AUTHORITY_STATUS};

#[derive(CreateCommand, SlashCommand)]
#[flags(SKIP_DEFER)]
#[command(name = "help")]
#[allow(dead_code)]
/// Display general help or help for a specific command
pub struct Help {
    #[command(autocomplete = true)]
    /// Specify a command base name
    command: Option<String>,
}

#[derive(CommandModel)]
#[command(autocomplete = true)]
struct Help_ {
    command: AutocompleteValue<String>,
}

pub async fn slash_help(ctx: Arc<Context>, mut command: InteractionCommand) -> Result<()> {
    let args = Help_::from_interaction(command.input_data())?;

    match args.command {
        AutocompleteValue::None => help_slash_basic(ctx, command).await,
        AutocompleteValue::Completed(name) => match SlashCommands::get().command(&name) {
            Some(cmd) => help_slash_command(&ctx, command, cmd).await,
            None => {
                let dists: BTreeMap<_, _> = SlashCommands::get()
                    .names()
                    .map(|cmd| (levenshtein_distance(&name, cmd).0, cmd))
                    .filter(|(dist, _)| *dist < 5)
                    .collect();

                let content = failed_message_content(dists);
                command.error_callback(&ctx, content).await?;

                Ok(())
            }
        },
        AutocompleteValue::Focused(name) => {
            let name = name.cow_to_ascii_lowercase();
            let arg = name.trim();

            let choices = match SlashCommands::get().descendants(arg) {
                Some(cmds) => cmds
                    .map(|cmd| CommandOptionChoice::String {
                        name: cmd.to_owned(),
                        name_localizations: None,
                        value: cmd.to_owned(),
                    })
                    .collect(),
                None => Vec::new(),
            };

            command.autocomplete(&ctx, choices).await?;

            Ok(())
        }
    }
}

async fn help_slash_basic(ctx: Arc<Context>, command: InteractionCommand) -> Result<()> {
    let id = ctx
        .cache
        .current_user(|user| user.id)
        .expect("missing CurrentUser in cache");

    let mention = format!("<@{id}>");

    let description = format!(
        "{mention} is a discord bot written by [mezo](https://osu.ppy.sh/u/13313647) \
        to render and upload replays"
    );

    let command_help = EmbedField {
        inline: false,
        name: "Want to learn more about a command?".to_owned(),
        value: "Try specifying the command name on the `help` command: `/help command:_`"
            .to_owned(),
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

    let github = EmbedField {
        inline: false,
        name: "Interested in the code?".to_owned(),
        value: format!("The source code can be found over at [github]({SHISHABOT_GITHUB})"),
    };

    let fields = vec![command_help, invite, servers, github];

    let builder = EmbedBuilder::new()
        .description(description)
        .fields(fields)
        .build()
        .into();

    command.callback(&ctx, builder, true).await?;

    Ok(())
}

async fn help_slash_command(
    ctx: &Context,
    command: InteractionCommand,
    cmd: &SlashCommand,
) -> Result<()> {
    let ApplicationCommandData {
        name,
        description,
        help,
        options,
        ..
    } = (cmd.create)();

    let description = help.unwrap_or(description);

    if name == "owner" {
        let description =
            "This command can only be used by the owners of the bot.\nQuit snooping around :^)";

        let embed_builder = EmbedBuilder::new().title(name).description(description);
        let builder = MessageBuilder::new().embed(embed_builder);
        command.callback(ctx, builder, true).await?;

        return Ok(());
    }

    let mut embed_builder = EmbedBuilder::new()
        .title(name)
        .description(description)
        .fields(option_fields(&options));

    if cmd.flags.authority() {
        let footer = FooterBuilder::new(AUTHORITY_STATUS);
        embed_builder = embed_builder.footer(footer);
    }

    let menu = parse_select_menu(&options);

    let builder = MessageBuilder::new()
        .embed(embed_builder)
        .components(menu.unwrap_or_default());

    command.callback(ctx, builder, true).await?;

    Ok(())
}
