use std::collections::HashSet;

use flurry::HashMap as FlurryMap;
use serde::{Deserialize, Serialize};
use twilight_model::id::{
    marker::{ChannelMarker, GuildMarker},
    Id,
};

use crate::util::hasher::SimpleBuildHasher;

type Servers = FlurryMap<Id<GuildMarker>, Server, SimpleBuildHasher>;

#[derive(Debug, Deserialize, Serialize)]
pub struct RootSettings {
    #[serde(rename = "Servers", with = "servers")]
    pub servers: Servers,
}

#[derive(Clone, Debug, Default)]
pub struct Server {
    pub input_channels: HashSet<Id<ChannelMarker>>,
    pub output_channel: Option<Id<ChannelMarker>>,
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

    use super::{FlurryMap, Server, Servers};

    #[derive(Deserialize)]
    struct RawServer {
        server_id: Id<GuildMarker>,
        input_channels: std::collections::HashSet<Id<ChannelMarker>>,
        output_channel: Option<Id<ChannelMarker>>,
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
                        input_channels,
                        output_channel,
                    } = raw;

                    let server = Server {
                        input_channels,
                        output_channel,
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
            let mut raw = s.serialize_struct("RawServer", 3)?;

            raw.serialize_field("server_id", &self.server_id)?;
            raw.serialize_field("input_channels", &self.server.input_channels)?;
            raw.serialize_field("output_channel", &self.server.output_channel)?;

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
