use twilight_model::{
    guild::Permissions,
    id::{
        marker::{ChannelMarker, GuildMarker, UserMarker},
        Id,
    },
};

use crate::core::Context;

/// Is authority -> None
/// No authority -> Some(message to user)
pub async fn check_authority(
    ctx: &Context,
    author: Id<UserMarker>,
    channel: Id<ChannelMarker>,
    guild: Option<Id<GuildMarker>>,
) -> Option<&'static str> {
    let permissions = ctx.cache.get_channel_permissions(author, channel, guild);

    if permissions.intersects(Permissions::ADMINISTRATOR | Permissions::MANAGE_CHANNELS) {
        None
    } else {
        let content =
            "You need admin permission or manage channels permission to use this command.";

        Some(content)
    }
}
