use std::collections::HashMap;

use serde::{
    de::{SeqAccess, Visitor},
    ser::SerializeSeq,
    Deserialize, Deserializer, Serialize, Serializer,
};
use serenity::model::id::{ChannelId, GuildId};

type Servers = HashMap<GuildId, Server>;

#[derive(Debug, Deserialize, Serialize)]
pub struct Root {
    #[serde(
        rename = "Servers",
        deserialize_with = "deserialize_servers",
        serialize_with = "serialize_servers"
    )]
    pub servers: Servers,
}

#[derive(Copy, Clone, Debug)]
pub struct Server {
    pub replay_channel: ChannelId,
    pub output_channel: ChannelId,
}

#[derive(Deserialize, Serialize)]
struct RawServer {
    server_id: GuildId,
    replay_channel: ChannelId,
    output_channel: ChannelId,
}

struct ServersVisitor;

impl<'de> Visitor<'de> for ServersVisitor {
    type Value = Servers;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("a list of servers")
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut servers = HashMap::with_capacity(seq.size_hint().unwrap_or(0));

        while let Some(RawServer {
            server_id,
            replay_channel,
            output_channel,
        }) = seq.next_element()?
        {
            let server = Server {
                replay_channel,
                output_channel,
            };

            servers.insert(server_id, server);
        }

        Ok(servers)
    }
}

fn deserialize_servers<'de, D: Deserializer<'de>>(d: D) -> Result<Servers, D::Error> {
    d.deserialize_seq(ServersVisitor)
}

fn serialize_servers<S: Serializer>(servers: &Servers, s: S) -> Result<S::Ok, S::Error> {
    let mut seq = s.serialize_seq(Some(servers.len()))?;

    for (&server_id, server) in servers.iter() {
        let server = RawServer {
            server_id,
            replay_channel: server.replay_channel,
            output_channel: server.output_channel,
        };

        seq.serialize_element(&server)?;
    }

    seq.end()
}
