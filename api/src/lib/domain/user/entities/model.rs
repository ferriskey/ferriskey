use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use utoipa::ToSchema;
use uuid::{NoContext, Timestamp, Uuid};

use crate::domain::user::dtos::user_dto::CreateUserDto;

#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd, FromRow, ToSchema,
)]
pub struct User {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub username: String,
    pub firstname: String,
    pub lastname: String,
    pub email: String,
    pub email_verified: bool,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct UserConfig {
    pub realm_id: Uuid,
    pub username: String,
    pub firstname: String,
    pub lastname: String,
    pub email: String,
    pub email_verified: bool,
    pub enabled: bool,
}

impl User {
    pub fn from_dto(dto: CreateUserDto) -> Self {
        let now = Utc::now();
        let seconds = now.timestamp().try_into().unwrap_or(0);
        let timestamp = Timestamp::from_unix(NoContext, seconds, 0);

        Self {
            id: Uuid::new_v7(timestamp),
            realm_id: dto.realm_id,
            username: dto.username,
            firstname: dto.firstname,
            lastname: dto.lastname,
            email: dto.email,
            email_verified: dto.email_verified,
            enabled: dto.enabled,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn new(user_config: UserConfig) -> Self {
        let now = Utc::now();
        let seconds = now.timestamp().try_into().unwrap_or(0);
        let timestamp = Timestamp::from_unix(NoContext, seconds, 0);

        Self {
            id: Uuid::new_v7(timestamp),
            realm_id: user_config.realm_id,
            username: user_config.username,
            firstname: user_config.firstname,
            lastname: user_config.lastname,
            email: user_config.email,
            email_verified: user_config.email_verified,
            enabled: user_config.enabled,
            created_at: now,
            updated_at: now,
        }
    }
}
