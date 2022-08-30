use flurry::HashMap as FlurryMap;
use serde::{Deserialize, Serialize};
use smallstr::SmallString;
use smallvec::SmallVec;
use twilight_model::id::{
    marker::{ChannelMarker, GuildMarker},
    Id,
};

use crate::util::hasher::SimpleBuildHasher;

pub type Prefix = SmallString<[u8; 2]>;
pub type Prefixes = SmallVec<[Prefix; 2]>;

type Servers = FlurryMap<Id<GuildMarker>, Server, SimpleBuildHasher>;

#[derive(Debug, Deserialize, Serialize)]
pub struct RootSettings {
    #[serde(rename = "Servers", with = "servers")]
    pub servers: Servers,
}

#[derive(Clone, Debug)]
pub struct Server {
    pub input_channel: Id<ChannelMarker>,
    pub output_channel: Id<ChannelMarker>,
    pub prefixes: Prefixes,
}

impl Server {
    pub fn new(input_channel: Id<ChannelMarker>, output_channel: Id<ChannelMarker>) -> Self {
        Self {
            input_channel,
            output_channel,
            prefixes: Prefixes::default(),
        }
    }
}

mod servers {
    use std::fmt::{Formatter, Result as FmtResult};

    use serde::{
        de::{SeqAccess, Visitor},
        ser::{SerializeSeq, SerializeStruct},
        Deserialize, Deserializer, Serialize, Serializer,
    };
    use twilight_model::id::{
        marker::{ChannelMarker, GuildMarker},
        Id,
    };

    use crate::util::hasher::SimpleBuildHasher;

    use super::{FlurryMap, Prefixes, Server, Servers};

    #[derive(Deserialize)]
    struct RawServer {
        server_id: Id<GuildMarker>,
        input_channel: Id<ChannelMarker>,
        output_channel: Id<ChannelMarker>,
        #[serde(default)]
        prefixes: Prefixes,
    }

    struct ServersVisitor;

    impl<'de> Visitor<'de> for ServersVisitor {
        type Value = Servers;

        fn expecting(&self, f: &mut Formatter<'_>) -> FmtResult {
            f.write_str("a list of servers")
        }

        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            let servers = FlurryMap::with_capacity_and_hasher(
                seq.size_hint().unwrap_or(0),
                SimpleBuildHasher,
            );

            {
                let guard = servers.pin();

                while let Some(raw) = seq.next_element()? {
                    let RawServer {
                        server_id,
                        input_channel,
                        output_channel,
                        prefixes,
                    } = raw;

                    let server = Server {
                        input_channel,
                        output_channel,
                        prefixes,
                    };

                    guard.insert(server_id, server);
                }
            }

            Ok(servers)
        }
    }

    pub(super) fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Servers, D::Error> {
        d.deserialize_seq(ServersVisitor)
    }

    struct BorrowedRawServer<'s> {
        server_id: Id<GuildMarker>,
        server: &'s Server,
    }

    impl Serialize for BorrowedRawServer<'_> {
        fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
            let mut raw =
                s.serialize_struct("RawServer", 4 - self.server.prefixes.is_empty() as usize)?;

            raw.serialize_field("server_id", &self.server_id)?;
            raw.serialize_field("input_channel", &self.server.input_channel)?;
            raw.serialize_field("output_channel", &self.server.output_channel)?;

            if !self.server.prefixes.is_empty() {
                raw.serialize_field("prefixes", &self.server.prefixes)?;
            }

            raw.end()
        }
    }

    pub(super) fn serialize<S: Serializer>(servers: &Servers, s: S) -> Result<S::Ok, S::Error> {
        let mut seq = s.serialize_seq(Some(servers.len()))?;

        for (&server_id, server) in servers.pin().iter() {
            let server = BorrowedRawServer { server_id, server };
            seq.serialize_element(&server)?;
        }

        seq.end()
    }
}
