use std::{hash::Hash, sync::Mutex};

use hashbrown::HashMap;
use time::OffsetDateTime;

use crate::util::hasher::SimpleBuildHasher;

pub struct Buckets([Mutex<Bucket>; 1]);

impl Buckets {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let make_bucket = |delay, time_span, limit| {
            let ratelimit = Ratelimit {
                delay,
                limit: Some((time_span, limit)),
            };

            Mutex::new(Bucket::new(ratelimit))
        };

        Self([
            make_bucket(0, 9, 4), // All
        ])
    }

    pub fn get(&self, bucket: BucketName) -> &Mutex<Bucket> {
        match bucket {
            BucketName::All => &self.0[0],
        }
    }
}

pub struct Ratelimit {
    pub delay: i64,
    pub limit: Option<(i64, i32)>,
}

#[derive(Default)]
pub struct MemberRatelimit {
    pub last_time: i64,
    pub set_time: i64,
    pub tickets: i32,
}

pub struct Bucket {
    pub ratelimit: Ratelimit,
    pub users: HashMap<u64, MemberRatelimit, SimpleBuildHasher>,
}

impl Bucket {
    fn new(ratelimit: Ratelimit) -> Self {
        Self {
            ratelimit,
            users: HashMap::default(),
        }
    }

    pub fn take(&mut self, user_id: u64) -> i64 {
        let time = OffsetDateTime::now_utc().unix_timestamp();

        let user = self
            .users
            .entry(user_id)
            .or_insert_with(MemberRatelimit::default);

        if let Some((timespan, limit)) = self.ratelimit.limit {
            if (user.tickets + 1) > limit {
                if time < (user.set_time + timespan) {
                    return (user.set_time + timespan) - time;
                } else {
                    user.tickets = 0;
                    user.set_time = time;
                }
            }
        }

        if time < user.last_time + self.ratelimit.delay {
            (user.last_time + self.ratelimit.delay) - time
        } else {
            user.tickets += 1;
            user.last_time = time;

            0
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub enum BucketName {
    All,
}
