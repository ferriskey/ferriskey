use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[typeshare]
pub struct JwtToken {
    access_token: String,
    token_type: String,
    refresh_token: String,
    expires_in: u32,
    id_token: String,
}

impl JwtToken {
    pub fn new(
        access_token: String,
        token_type: String,
        refresh_token: String,
        expires_in: u32,
        id_token: String,
    ) -> Self {
        Self {
            access_token,
            token_type,
            refresh_token,
            expires_in,
            id_token,
        }
    }
}
