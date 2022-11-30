use sqlx::FromRow;

#[derive(Debug, FromRow)]
#[sqlx(type_name = "blacklisted_servers")]
pub struct DBServerBlacklist {
    pub guild_id: u128,
    pub reason: Option<String>,
}
