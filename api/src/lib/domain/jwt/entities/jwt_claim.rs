use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, PartialOrd, Ord, ToSchema)]
pub struct JwtClaim {
    pub sub: String,
    pub exp: i64,
    pub iss: String,
    pub aud: Vec<String>,
    pub typ: String,
    pub azp: String,
}

impl JwtClaim {
    pub fn new(sub: String, iss: String, aud: Vec<String>, typ: String, azp: String) -> Self {
        Self {
            sub,
            exp: chrono::Utc::now().timestamp() + 3600,
            iss,
            aud,
            typ,
            azp,
        }
    }
}
