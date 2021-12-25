use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize)]
pub struct Root {
    #[serde(rename = "Servers")]
    pub servers: Vec<Server>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct Server {
    #[serde(rename = "server_id")]
    pub server_id: String,
    #[serde(rename = "replay_channel")]
    pub replay_channel: String,
    #[serde(rename = "output_channel")]
    pub output_channel: String,
}
