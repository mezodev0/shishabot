use std::{collections::BTreeMap, fmt::Write, sync::Arc};

use command_macros::command;
use eyre::{ContextCompat, Report, Result};
use hashbrown::HashSet;
use twilight_model::{
    application::component::{select_menu::SelectMenuOption, ActionRow, Component, SelectMenu},
    channel::{embed::EmbedField, Message, ReactionType},
    id::{marker::GuildMarker, Id},
};

use crate::{
    core::{
        commands::prefix::{PrefixCommand, PrefixCommandGroup, PrefixCommands},
        Context,
    },
    util::{
        builder::{AuthorBuilder, EmbedBuilder, FooterBuilder, MessageBuilder},
        interaction::InteractionComponent,
        levenshtein_distance, ChannelExt, ComponentExt,
    },
};

use super::failed_message_content;

#[command]
#[desc("Display help for prefix commands")]
#[group(Utility)]
#[alias("h")]
#[usage("[command]")]
#[example("", "recent", "osg")]
async fn prefix_help(ctx: Arc<Context>, msg: &Message, mut args: Args<'_>) -> Result<()> {
    match args.next() {
        Some(arg) => match PrefixCommands::get().command(arg) {
            Some(cmd) => command_help(ctx, msg, cmd).await,
            None => failed_help(ctx, msg, arg).await,
        },
        None => dm_help(ctx, msg).await,
    }
}

async fn failed_help(ctx: Arc<Context>, msg: &Message, name: &str) -> Result<()> {
    let mut seen = HashSet::new();

    let dists: BTreeMap<_, _> = PrefixCommands::get()
        .iter()
        .filter(|cmd| seen.insert(cmd.name()))
        .flat_map(|cmd| cmd.names.iter())
        .map(|&cmd| (levenshtein_distance(name, cmd).0, cmd))
        .filter(|(dist, _)| *dist < 4)
        .collect();

    let content = failed_message_content(dists);
    msg.error(&ctx, content).await?;

    Ok(())
}

async fn command_help(ctx: Arc<Context>, msg: &Message, cmd: &PrefixCommand) -> Result<()> {
    let name = cmd.name();
    let prefix = ctx.guild_first_prefix(msg.guild_id).await;
    let mut fields = Vec::new();

    let mut eb = EmbedBuilder::new()
        .title(name)
        .description(cmd.help.unwrap_or(cmd.desc));

    let mut usage_len = 0;

    if let Some(usage) = cmd.usage {
        let value = format!("`{prefix}{name} {usage}`");
        usage_len = value.chars().count();

        let field = EmbedField {
            name: "How to use".to_owned(),
            value,
            inline: usage_len <= 29,
        };

        fields.push(field);
    }

    let mut examples = cmd.examples.iter();

    if let Some(first) = examples.next() {
        let len: usize = cmd.examples.iter().map(|&e| name.len() + e.len() + 4).sum();
        let mut value = String::with_capacity(len);
        let mut example_len = 0;
        let cmd_len = prefix.chars().count() + name.chars().count();
        writeln!(value, "`{prefix}{name} {first}`")?;

        for example in examples {
            writeln!(value, "`{prefix}{name} {example}`")?;
            example_len = example_len.max(cmd_len + example.chars().count());
        }

        let not_inline = (usage_len <= 29 && cmd.names.len() > 1 && example_len > 27)
            || ((usage_len > 29 || cmd.names.len() > 1) && example_len > 36)
            || example_len > 45;

        let field = EmbedField {
            name: "Examples".to_owned(),
            value,
            inline: !not_inline,
        };

        fields.push(field);
    }

    let mut aliases = cmd.names.iter().skip(1);

    if let Some(first) = aliases.next() {
        let len: usize = cmd.names.iter().skip(1).map(|n| 4 + n.len()).sum();
        let mut value = String::with_capacity(len);
        write!(value, "`{first}`")?;

        for &alias in aliases {
            write!(value, ", `{alias}`")?;
        }

        let field = EmbedField {
            name: "Aliases".to_owned(),
            value,
            inline: true,
        };

        fields.push(field);
    }

    if cmd.flags.authority() {
        let field = EmbedField {
            name: "Requires authority status".to_owned(),
            value: "Admin permission or manage channels permission".to_owned(),
            inline: false,
        };

        fields.push(field);
    }

    if cmd.flags.only_owner() {
        let author = AuthorBuilder::new("Can only be used by the bot owner");
        eb = eb.author(author);
    }

    let footer_text = if cmd.flags.only_guilds() || cmd.flags.authority() {
        "Only available in servers"
    } else {
        "Available in servers and DMs"
    };

    let footer = FooterBuilder::new(footer_text);

    let embed = eb.footer(footer).fields(fields).build();
    let builder = MessageBuilder::new().embed(embed);

    msg.create_message(&ctx, &builder).await?;

    Ok(())
}

async fn description(ctx: &Context, guild_id: Option<Id<GuildMarker>>) -> String {
    format!("Bla bla TODO write this mezo")
}

async fn dm_help(ctx: Arc<Context>, msg: &Message) -> Result<()> {
    let owner = msg.author.id;

    let channel = match ctx.http.create_private_channel(owner).exec().await {
        Ok(channel_res) => channel_res.model().await?.id,
        Err(err) => {
            let content = "Your DMs seem blocked :(\n\
            Perhaps you disabled incoming messages from other server members?";
            let report = Report::new(err).wrap_err("error while creating DM channel");
            warn!("{report:?}");
            msg.error(&ctx, content).await?;

            return Ok(());
        }
    };

    if msg.guild_id.is_some() {
        let content = "Don't mind me sliding into your DMs :eyes:";
        let builder = MessageBuilder::new().embed(content);
        let _ = msg.create_message(&ctx, &builder).await;
    }

    let desc = description(&ctx, msg.guild_id).await;
    let embed = EmbedBuilder::new().description(desc).build();
    let components = help_select_menu(None);
    let builder = MessageBuilder::new().embed(embed).components(components);

    if let Err(err) = channel.create_message(&ctx, &builder).await {
        let report = Report::new(err).wrap_err("error while sending help chunk");
        warn!("{report:?}");
        let content = "Could not DM you, perhaps you disabled it?";
        msg.error(&ctx, content).await?;
    }

    Ok(())
}

pub async fn handle_help_category(
    ctx: &Context,
    mut component: InteractionComponent,
) -> Result<()> {
    let value = component.data.values.pop().context("missing menu value")?;

    let group = match value.as_str() {
        "general" => {
            let desc = description(ctx, None).await;
            let embed = EmbedBuilder::new().description(desc).build();
            let components = help_select_menu(None);
            let builder = MessageBuilder::new().embed(embed).components(components);

            component.callback(ctx, builder).await?;

            return Ok(());
        }
        "osu" => PrefixCommandGroup::Osu,
        "taiko" => PrefixCommandGroup::Taiko,
        "ctb" => PrefixCommandGroup::Catch,
        "mania" => PrefixCommandGroup::Mania,
        "all_modes" => PrefixCommandGroup::AllModes,
        "tracking" => PrefixCommandGroup::Tracking,
        "twitch" => PrefixCommandGroup::Twitch,
        "games" => PrefixCommandGroup::Games,
        "danser" => PrefixCommandGroup::Danser,
        "utility" => PrefixCommandGroup::Utility,
        _ => bail!("got unexpected value `{value}`"),
    };

    let mut cmds: Vec<_> = {
        let mut dedups = HashSet::new();

        PrefixCommands::get()
            .iter()
            .filter(|cmd| cmd.group == group)
            .filter(|cmd| dedups.insert(cmd.name()))
            .collect()
    };

    cmds.sort_unstable_by_key(|cmd| cmd.name());

    let mut desc = String::with_capacity(64);

    let emote = group.emote();
    let name = group.name();
    let _ = writeln!(desc, "{emote} __**{name}**__");

    for cmd in cmds {
        let name = cmd.name();
        let authority = if cmd.flags.authority() { "**\\***" } else { "" };
        let _ = writeln!(desc, "`{name}`{authority}: {}", cmd.desc);
    }

    let footer = FooterBuilder::new(
        "*: Either can't be used in DMs or requires authority status in the server",
    );

    let embed = EmbedBuilder::new().description(desc).footer(footer).build();
    let components = help_select_menu(Some(group));
    let builder = MessageBuilder::new().embed(embed).components(components);

    component.callback(ctx, builder).await?;

    Ok(())
}

fn help_select_menu(default: Option<PrefixCommandGroup>) -> Vec<Component> {
    let options = vec![
        SelectMenuOption {
            default: matches!(default, None),
            description: None,
            emoji: Some(ReactionType::Unicode {
                name: "🛁".to_owned(),
            }),
            label: "General".to_owned(),
            value: "general".to_owned(),
        },
        SelectMenuOption {
            default: matches!(default, Some(PrefixCommandGroup::Osu)),
            description: None,
            emoji: None,
            label: "osu!".to_owned(),
            value: "osu".to_owned(),
        },
        SelectMenuOption {
            default: matches!(default, Some(PrefixCommandGroup::Taiko)),
            description: None,
            emoji: None,
            label: "Taiko".to_owned(),
            value: "taiko".to_owned(),
        },
        SelectMenuOption {
            default: matches!(default, Some(PrefixCommandGroup::Catch)),
            description: None,
            emoji: None,
            label: "Catch".to_owned(),
            value: "ctb".to_owned(),
        },
        SelectMenuOption {
            default: matches!(default, Some(PrefixCommandGroup::Mania)),
            description: None,
            emoji: None,
            label: "Mania".to_owned(),
            value: "mania".to_owned(),
        },
        SelectMenuOption {
            default: matches!(default, Some(PrefixCommandGroup::AllModes)),
            description: None,
            emoji: None,
            label: "All Modes".to_owned(),
            value: "all_modes".to_owned(),
        },
        SelectMenuOption {
            default: matches!(default, Some(PrefixCommandGroup::Tracking)),
            description: None,
            emoji: None,
            label: "Tracking".to_owned(),
            value: "tracking".to_owned(),
        },
        SelectMenuOption {
            default: matches!(default, Some(PrefixCommandGroup::Twitch)),
            description: None,
            emoji: None,
            label: "Twitch".to_owned(),
            value: "twitch".to_owned(),
        },
        SelectMenuOption {
            default: matches!(default, Some(PrefixCommandGroup::Games)),
            description: None,
            emoji: None,
            label: "Games".to_owned(),
            value: "games".to_owned(),
        },
        SelectMenuOption {
            default: matches!(default, Some(PrefixCommandGroup::Danser)),
            description: None,
            emoji: None,
            label: "Danser".to_owned(),
            value: "danser".to_owned(),
        },
        SelectMenuOption {
            default: matches!(default, Some(PrefixCommandGroup::Utility)),
            description: None,
            emoji: None,
            label: "Utility".to_owned(),
            value: "utility".to_owned(),
        },
    ];

    let category = SelectMenu {
        custom_id: "help_category".to_owned(),
        disabled: false,
        max_values: Some(1),
        min_values: Some(1),
        options,
        placeholder: None,
    };

    let category_row = ActionRow {
        components: vec![Component::SelectMenu(category)],
    };

    vec![Component::ActionRow(category_row)]
}
