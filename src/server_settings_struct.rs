use std::collections::HashMap;

use serde::{
    de::{SeqAccess, Visitor},
    ser::{SerializeSeq, SerializeStruct},
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

#[derive(Clone, Debug)]
pub struct Server {
    pub replay_channel: ChannelId,
    pub output_channel: ChannelId,
    pub prefixes: Vec<String>,
}

#[derive(Deserialize)]
struct RawServer {
    server_id: GuildId,
    replay_channel: ChannelId,
    output_channel: ChannelId,
    #[serde(default)]
    prefixes: Vec<String>,
}

struct ServersVisitor;

impl<'de> Visitor<'de> for ServersVisitor {
    type Value = Servers;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("a list of servers")
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut servers = HashMap::with_capacity(seq.size_hint().unwrap_or(0));

        while let Some(raw) = seq.next_element()? {
            let RawServer {
                server_id,
                replay_channel,
                output_channel,
                prefixes,
            } = raw;

            let server = Server {
                replay_channel,
                output_channel,
                prefixes,
            };

            servers.insert(server_id, server);
        }

        Ok(servers)
    }
}

fn deserialize_servers<'de, D: Deserializer<'de>>(d: D) -> Result<Servers, D::Error> {
    d.deserialize_seq(ServersVisitor)
}

struct BorrowedRawServer<'s> {
    server_id: GuildId,
    server: &'s Server,
}

impl Serialize for BorrowedRawServer<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut raw =
            s.serialize_struct("RawServer", 4 - self.server.prefixes.is_empty() as usize)?;

        raw.serialize_field("server_id", &self.server_id)?;
        raw.serialize_field("replay_channel", &self.server.replay_channel)?;
        raw.serialize_field("output_channel", &self.server.output_channel)?;

        if !self.server.prefixes.is_empty() {
            raw.serialize_field("prefixes", &self.server.prefixes)?;
        }

        raw.end()
    }
}

fn serialize_servers<S: Serializer>(servers: &Servers, s: S) -> Result<S::Ok, S::Error> {
    let mut seq = s.serialize_seq(Some(servers.len()))?;

    for (&server_id, server) in servers.iter() {
        let server = BorrowedRawServer { server_id, server };
        seq.serialize_element(&server)?;
    }

    seq.end()
}
