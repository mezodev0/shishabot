use serenity::{
    client::Context,
    framework::standard::{macros::check, Args, CommandOptions, Reason},
    model::{channel::Message, Permissions},
};

#[check]
#[name = "Permissions"]
async fn permissions_check(
    ctx: &Context,
    msg: &Message,
    _: &mut Args,
    _: &CommandOptions,
) -> Result<(), Reason> {
    let guild_id = msg
        .guild_id
        .ok_or_else(|| Reason::User("message was not sent in a guild".to_string()))?;

    if let Some(guild) = ctx.cache.guild(guild_id) {
        info!("guild passed");

        let channels = match guild.channels(&ctx).await {
            Ok(channels) => channels,
            Err(err) => return Err(Reason::User(format!("couldn't fetch channels: {err}"))),
        };
        info!("channels passed");

        let mut guild_channel = match &ctx.cache.guild_channel(&msg.channel_id) {
            Some(channel) => channel.to_owned(),
            None => todo!(),
        };

        for (id, channel) in channels.into_iter() {
            if id == msg.channel_id {
                guild_channel = channel;
                info!("channel found");
                break;
            }
        }

        // if let Some(channel) = &ctx.cache.guild_channel(msg.channel_id) {
        let member = match guild_id.member(&ctx, &msg.author.id).await {
            Ok(member) => member,
            Err(err) => return Err(Reason::User(format!("couldn't fetch member: {err}"))),
        };
        info!("member passed");
        let perms = match guild.user_permissions_in(&guild_channel, &member) {
            Ok(perms) => perms,
            Err(err) => return Err(Reason::User(format!("couldn't fetch permissions: {err}"))),
        };
        info!("perms passed");
        if !perms
            .intersection(Permissions::ADMINISTRATOR | Permissions::MANAGE_CHANNELS)
            .is_empty()
        {
            info!("ok");
            return Ok(());
        }
        // }
    }
    info!("err");
    Err(Reason::User(
        "Lacking required permission to run command".to_string(),
    ))
}
