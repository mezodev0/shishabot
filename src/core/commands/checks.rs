use twilight_model::{
    guild::Permissions,
    id::{
        marker::{GuildMarker, UserMarker},
        Id,
    },
};

use crate::core::{buckets::BucketName, Context};

/// Is authority -> None
/// No authority -> Some(message to user)
pub async fn check_authority(
    ctx: &Context,
    author: Id<UserMarker>,
    guild: Option<Id<GuildMarker>>,
) -> Option<String> {
    let (guild_id, (permissions, roles)) = match guild {
        Some(guild) => (guild, ctx.cache.get_guild_permissions(author, guild)),
        None => return None,
    };

    if permissions.contains(Permissions::ADMINISTRATOR | Permissions::MANAGE_CHANNELS) {
        None
    } else {
        let content =
            "You need admin permission or manage channels permission to use this command.";

        Some(content.to_owned())
    }
}

pub async fn check_ratelimit(
    ctx: &Context,
    user: Id<UserMarker>,
    bucket: BucketName,
) -> Option<i64> {
    let ratelimit = ctx.buckets.get(bucket).lock().take(user.get());

    (ratelimit > 0).then(|| ratelimit)
}
