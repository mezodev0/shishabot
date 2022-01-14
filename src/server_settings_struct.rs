use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize)]
pub struct Root {
    #[serde(rename = "Servers")]
    pub servers: Vec<Server>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct Server {
    pub server_id: String,
    pub replay_channel: String,
    pub output_channel: String,
}
