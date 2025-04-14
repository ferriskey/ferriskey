use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, PartialOrd, Ord, ToSchema)]
pub struct JwtClaim {
    pub sub: Uuid,
    pub preferred_username: String,
    pub exp: i64,
    pub iss: String,
    pub aud: Vec<String>,
    pub typ: String,
    pub azp: String,
    pub client_id: Option<String>,
}

impl JwtClaim {
    pub fn new(
        sub: Uuid,
        preferred_username: String,
        iss: String,
        aud: Vec<String>,
        typ: String,
        azp: String,
    ) -> Self {
        Self {
            sub,
            preferred_username,
            exp: chrono::Utc::now().timestamp() + 3600,
            iss,
            aud,
            typ,
            azp,
            client_id: None,
        }
    }
}
