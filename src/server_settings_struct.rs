use serde::{Deserialize, Serialize};
use serenity::model::id::GuildId;

#[derive(Deserialize, Debug, Serialize)]
pub struct Root {
    #[serde(rename = "Servers")]
    pub servers: Vec<Server>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct Server {
    pub server_id: GuildId,
    pub replay_channel: String,
    pub output_channel: String,
}
