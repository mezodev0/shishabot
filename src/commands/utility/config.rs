use std::sync::Arc;

use command_macros::{command, SlashCommand};
use rosu_v2::prelude::GameMode;
use twilight_interactions::command::{CommandModel, CommandOption, CreateCommand, CreateOption};

use crate::{
    commands::ShowHideOption,
    embeds::{ConfigEmbed, EmbedData},
    util::{
        constants::GENERAL_ISSUE, interaction::InteractionCommand, Authored, InteractionCommandExt,
    },
    BotResult, Context,
};

#[derive(CommandModel, CreateCommand, Default, SlashCommand)]
#[command(name = "config")]
#[flags(EPHEMERAL)]
/// Adjust your default configuration for commands
pub struct Config {
    #[command(help = "Always having to specify the `mode` option for any non-std \
    command can be pretty tedious.\nTo get around that, you can configure a mode here so \
    that when the `mode` option is not specified in commands, it will choose your config mode.")]
    /// Specify a gamemode (NOTE: Only use for non-std modes if you NEVER use std commands)
    mode: Option<ConfigGameMode>,
    #[command(help = "Some embeds are pretty chunky and show too much data.\n\
    With this option you can make those embeds minimized by default.\n\
    Affected commands are: `compare score`, `recent score`, `recent simulate`, \
    and any command showing top scores when the `index` option is specified.")]
    /// What size should the recent, compare, simulate, ... commands be?
    score_embeds: Option<ConfigEmbeds>,
    /// Should the amount of retries be shown for the recent command?
    retries: Option<ShowHideOption>,
}

#[derive(CommandOption, CreateOption)]
pub enum ConfigGameMode {
    #[option(name = "None", value = "none")]
    None,
    #[option(name = "osu", value = "osu")]
    Osu,
    #[option(name = "taiko", value = "taiko")]
    Taiko,
    #[option(name = "ctb", value = "ctb")]
    Catch,
    #[option(name = "mania", value = "mania")]
    Mania,
}

impl From<ConfigGameMode> for Option<GameMode> {
    fn from(mode: ConfigGameMode) -> Self {
        match mode {
            ConfigGameMode::None => None,
            ConfigGameMode::Osu => Some(GameMode::Osu),
            ConfigGameMode::Taiko => Some(GameMode::Taiko),
            ConfigGameMode::Catch => Some(GameMode::Catch),
            ConfigGameMode::Mania => Some(GameMode::Mania),
        }
    }
}

#[derive(CommandOption, CreateOption)]
pub enum ConfigEmbeds {
    #[option(name = "Initial maximized", value = "initial_max")]
    InitialMax,
    #[option(name = "Always maximized", value = "max")]
    AlwaysMax,
    #[option(name = "Always minimized", value = "min")]
    AlwaysMin,
}

#[derive(CommandOption, CreateOption)]
pub enum ConfigMinimizedPp {
    #[option(name = "Max PP", value = "max")]
    MaxPp,
    #[option(name = "If FC", value = "if_fc")]
    IfFc,
}

async fn slash_config(ctx: Arc<Context>, mut command: InteractionCommand) -> BotResult<()> {
    let args = Config::from_interaction(command.input_data())?;

    config(ctx, command, args).await
}

pub async fn config(
    ctx: Arc<Context>,
    command: InteractionCommand,
    config: Config,
) -> BotResult<()> {
    let Config {
        mode,
        score_embeds,
        retries,
    } = config;

    let author = command.user_id()?;

    let mut config = match ctx.psql().get_user_config(author).await {
        Ok(Some(config)) => config,
        Ok(None) => UserConfig::default(),
        Err(err) => {
            let _ = command.error(&ctx, GENERAL_ISSUE).await;

            return Err(err);
        }
    };

    match mode {
        None => {}
        Some(ConfigGameMode::None) => config.mode = None,
        Some(ConfigGameMode::Osu) => config.mode = Some(GameMode::Osu),
        Some(ConfigGameMode::Taiko) => config.mode = Some(GameMode::Taiko),
        Some(ConfigGameMode::Catch) => config.mode = Some(GameMode::Catch),
        Some(ConfigGameMode::Mania) => config.mode = Some(GameMode::Mania),
    }

    if let Some(score_embeds) = score_embeds {
        config.score_size = Some(score_embeds.into());
    }

    if let Some(retries) = retries {
        config.show_retries = Some(matches!(retries, ShowHideOption::Show));
    }

    handle_no_links(&ctx, command, config).await
}

async fn handle_no_links(
    ctx: &Context,
    command: InteractionCommand,
    mut config: UserConfig,
) -> BotResult<()> {
    let author = command.user()?;

    if let Err(err) = ctx.psql().insert_user_config(author.id, &config).await {
        let _ = command.error(ctx, GENERAL_ISSUE).await;

        return Err(err);
    }

    let embed_data = ConfigEmbed::new(author, config, None);
    let builder = embed_data.build().into();
    command.update(ctx, &builder).await?;

    Ok(())
}
