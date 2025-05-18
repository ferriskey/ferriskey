use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, ToSchema)]
pub enum ClaimsTyp {
    Refresh,
    Bearer,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, PartialOrd, Ord, ToSchema)]
pub struct JwtClaim {
    pub sub: Uuid,
    pub iat: i64,
    pub jti: Uuid,
    pub iss: String,
    pub typ: ClaimsTyp,
    pub azp: String,
    pub aud: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub exp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
}

impl JwtClaim {
    pub fn new(
        sub: Uuid,
        preferred_username: String,
        iss: String,
        aud: Vec<String>,
        typ: ClaimsTyp,
        azp: String,
    ) -> Self {
        Self {
            sub,
            preferred_username: Some(preferred_username),
            iat: chrono::Utc::now().timestamp(),
            jti: Uuid::new_v4(),
            exp: Some(chrono::Utc::now().timestamp() + 3600), // 1 hour
            iss,
            aud,
            typ,
            azp,
            client_id: None,
        }
    }

    pub fn new_refresh_token(sub: Uuid, iss: String, aud: Vec<String>, azp: String) -> Self {
        Self {
            sub,
            iat: chrono::Utc::now().timestamp(),
            jti: Uuid::new_v4(),
            iss,
            aud,
            typ: ClaimsTyp::Refresh,
            azp,
            preferred_username: None,
            exp: Some(chrono::Utc::now().timestamp() + 86400), // 24 hours
            client_id: None,
        }
    }

    pub fn is_service_account(&self) -> bool {
        self.client_id.is_some()
    }
}
