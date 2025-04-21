use argon2::password_hash::rand_core::{OsRng, RngCore};
use base64_url::encode;
use chrono::{DateTime, Utc};
use uuid::{NoContext, Timestamp, Uuid};

pub fn generate_timestamp() -> (DateTime<Utc>, Timestamp) {
    let now = Utc::now();
    let seconds = now.timestamp().try_into().unwrap_or(0);

    let timestamp = Timestamp::from_unix(NoContext, seconds, 0);

    (now, timestamp)
}

pub fn generate_uuid_v7() -> Uuid {
    let (_, timestamp) = generate_timestamp();
    Uuid::new_v7(timestamp)
}

pub fn generate_random_string() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    encode(&bytes)
}
