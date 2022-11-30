use eyre::{Result, WrapErr};
use sqlx::Row;
use twilight_model::id::{marker::GuildMarker, Id};

use crate::database::Database;

impl Database {
    pub async fn blacklist_server(&self, guild_id: u64, reason: Option<String>) -> Result<()> {
        let query = sqlx::query(
            "
INSERT INTO blacklisted_servers (guild_id, reason) 
VALUES 
  ($1, $2) ON CONFLICT (guild_id) 
  DO 
UPDATE 
SET 
  reason = EXCLUDED.reason",
        );

        query
            .bind(guild_id as i64)
            .bind(reason)
            .execute(&self.pool)
            .await
            .wrap_err("failed store blacklisted server")?;

        Ok(())
    }

    pub async fn _is_server_blacklisted(&self, guild_id: Id<GuildMarker>) -> Result<bool> {
        let query = sqlx::query(
            "
SELECT 
  EXISTS (
    SELECT 
    FROM 
      blacklisted_servers 
    WHERE 
      guild_id = $1
  )",
        );

        let row = query
            .bind(guild_id.get() as i64)
            .fetch_one(&self.pool)
            .await
            .wrap_err("failed to check if server is blacklisted")?;

        Ok(row.get("exists"))
    }
}
