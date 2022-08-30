use std::sync::Arc;

use command_macros::{command, SlashCommand};
use twilight_cache_inmemory::model::CachedGuild;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
    id::{marker::GuildMarker, Id},
    util::ImageHash,
};

use crate::{
    commands::{EnableDisable, ShowHideOption},
    embeds::{EmbedData, ServerConfigEmbed},
    util::{constants::GENERAL_ISSUE, interaction::InteractionCommand, InteractionCommandExt},
    BotResult, Context,
};

pub struct GuildData {
    pub icon: Option<ImageHash>,
    pub id: Id<GuildMarker>,
    pub name: String,
}

impl From<&CachedGuild> for GuildData {
    #[inline]
    fn from(guild: &CachedGuild) -> Self {
        Self {
            icon: guild.icon().map(ImageHash::to_owned),
            id: guild.id(),
            name: guild.name().to_owned(),
        }
    }
}

#[derive(CommandModel, CreateCommand, SlashCommand)]
#[command(name = "serverconfig")]
#[flags(AUTHORITY, ONLY_GUILDS, SKIP_DEFER)]
/// Adjust configurations for a server
pub struct ServerConfig {
    /// Choose whether song commands can be used or not
    song_commands: Option<EnableDisable>,
    #[command(
        help = "Should the amount of retries be shown for the `recent` command?\n\
        Applies only if the member has not specified a config for themselves."
    )]
    /// Should the amount of retries be shown for the recent command?
    retries: Option<ShowHideOption>,
    #[command(
        min_value = 1,
        max_value = 100,
        help = "Specify the default track limit for tracking user's osu! top scores.\n\
        The value must be between 1 and 100, defaults to 50."
    )]
    /// Specify the default track limit for osu! top scores
    track_limit: Option<i64>,
}

impl ServerConfig {
    fn any(&self) -> bool {
        self.song_commands.is_some() || self.retries.is_some() || self.track_limit.is_some()
    }
}

async fn slash_serverconfig(ctx: Arc<Context>, mut command: InteractionCommand) -> BotResult<()> {
    let args = ServerConfig::from_interaction(command.input_data())?;

    let guild_id = command.guild_id.unwrap();

    let guild = match ctx.cache.guild(guild_id, |guild| guild.into()) {
        Ok(guild) => guild,
        Err(err) => {
            let _ = command.error(&ctx, GENERAL_ISSUE).await;

            return Err(err.into());
        }
    };

    if args.any() {
        let f = |config: &mut GuildConfig| {
            let ServerConfig {
                retries,
                song_commands,
                track_limit,
            } = args;

            if let Some(retries) = retries {
                config.show_retries = Some(retries == ShowHideOption::Show);
            }

            if let Some(limit) = track_limit {
                config.track_limit = Some(limit as u8);
            }

            if let Some(with_lyrics) = song_commands {
                config.with_lyrics = Some(with_lyrics == EnableDisable::Enable);
            }
        };

        if let Err(err) = ctx.update_guild_config(guild_id, f).await {
            let _ = command.error_callback(&ctx, GENERAL_ISSUE).await;

            return Err(err);
        }
    }

    let config = ctx.guild_config(guild_id).await;
    let mut authorities = Vec::with_capacity(config.authorities.len());

    for &auth in &config.authorities {
        if let Some(Ok(name)) =
            Id::new_checked(auth).map(|role| ctx.cache.role(role, |role| role.name.to_owned()))
        {
            authorities.push(name);
        }
    }

    let embed = ServerConfigEmbed::new(guild, config, &authorities);
    let builder = embed.build().into();
    command.callback(&ctx, builder, false).await?;

    Ok(())
}
