use serenity::{
    client::Context,
    framework::standard::{macros::check, Args, CommandOptions, Reason},
    model::{
        channel::{Channel, Message},
        Permissions,
    },
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
        .ok_or_else(|| Reason::Log("message was not sent in a guild".to_string()))?;

    if let Some(guild) = ctx.cache.guild(guild_id) {
        let guild_channel = match msg.channel_id.to_channel(&ctx).await {
            Ok(Channel::Guild(channel)) => channel,
            Err(err) => return Err(Reason::Log(format!("couldn't fetch channel: {:?}", err))),
            Ok(_) => todo!("private or group channel"),
        };

        // if let Some(channel) = &ctx.cache.guild_channel(msg.channel_id) {
        let member = match guild_id.member(&ctx, &msg.author.id).await {
            Ok(member) => member,
            Err(err) => return Err(Reason::Log(format!("couldn't fetch member: {:?}", err))),
        };

        let perms = match guild.user_permissions_in(&guild_channel, &member) {
            Ok(perms) => perms,
            Err(err) => {
                return Err(Reason::Log(format!(
                    "couldn't fetch permissions: {:?}",
                    err
                )))
            }
        };

        if !perms
            .intersection(Permissions::ADMINISTRATOR | Permissions::MANAGE_CHANNELS)
            .is_empty()
        {
            return Ok(());
        }
    }
    Err(Reason::User(
        "Lacking required permission to run command".to_string(),
    ))
}
