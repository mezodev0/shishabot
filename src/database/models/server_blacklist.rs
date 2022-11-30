use sqlx::FromRow;

#[derive(Debug, FromRow)]
#[sqlx(type_name = "blacklisted_servers")]
pub struct DBServerBlacklist {
    pub server_id: u64,
    pub reason: Option<String>,
}
