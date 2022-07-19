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
        .ok_or_else(|| Reason::Log("message was not sent in a guild".to_string()))?;

    let guild = ctx.cache.guild(guild_id).unwrap();

    let perms = guild
        .user_permissions_in(
            &ctx.cache.guild_channel(msg.channel_id).unwrap(),
            &msg.guild_id
                .unwrap()
                .member(&ctx, msg.author.id)
                .await
                .unwrap(),
        )
        .unwrap();

    if perms.contains(Permissions::ADMINISTRATOR) || perms.contains(Permissions::MANAGE_CHANNELS) {
        return Ok(());
    }

    Err(Reason::User(
        "Lacking required permission to run command".to_string(),
    ))
}
