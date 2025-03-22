use chrono::{DateTime, Utc};
use uuid::{NoContext, Timestamp};

pub fn generate_timestamp() -> (DateTime<Utc>, Timestamp) {
    let now = Utc::now();
    let seconds = now.timestamp().try_into().unwrap_or(0);

    let timestamp = Timestamp::from_unix(NoContext, seconds, 0);

    (now, timestamp)
}
