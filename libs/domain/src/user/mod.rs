use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    generate_uuid_v7,
    realm::{Realm, RealmId},
    role::Role,
    user::required_action::RequiredAction,
};

pub mod inputs;
pub mod ports;
pub mod required_action;
pub mod value_objects;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct User {
    pub id: Uuid,
    pub realm_id: RealmId,
    pub client_id: Option<Uuid>,
    pub username: String,
    pub firstname: String,
    pub lastname: String,
    pub email: String,
    pub email_verified: bool,
    pub enabled: bool,
    pub roles: Vec<Role>,
    pub realm: Option<Realm>,
    pub required_actions: Vec<RequiredAction>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct UserConfig {
    pub realm_id: RealmId,
    pub client_id: Option<Uuid>,
    pub username: String,
    pub firstname: String,
    pub lastname: String,
    pub email: String,
    pub email_verified: bool,
    pub enabled: bool,
}

impl User {
    pub fn new(user_config: UserConfig) -> Self {
        let id = generate_uuid_v7();
        let now = Utc::now();

        Self {
            id,
            realm_id: user_config.realm_id,
            client_id: user_config.client_id,
            username: user_config.username,
            firstname: user_config.firstname,
            lastname: user_config.lastname,
            email: user_config.email,
            email_verified: user_config.email_verified,
            enabled: user_config.enabled,
            roles: Vec::new(),
            realm: None,
            required_actions: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
}
