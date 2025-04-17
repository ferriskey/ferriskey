use uuid::Uuid;

use crate::domain::authentication::entities::{error::AuthenticationError, jwt_token::JwtToken};

pub struct GrantTypeParams {
    pub realm_id: Uuid,
    pub realm_name: String,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub code: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub refresh_token: Option<String>,
    pub redirect_uri: Option<String>,
}

pub trait GrantTypeStrategy: Clone + Send + Sync + 'static {
    fn execute(
        &self,
        params: GrantTypeParams,
    ) -> impl Future<Output = Result<JwtToken, AuthenticationError>> + Send;
}
