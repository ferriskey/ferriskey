use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{
    authentication::entities::{
        dto::AuthenticateDto, error::AuthenticationError, jwt_token::JwtToken,
    },
    user::entities::required_action::RequiredAction,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthenticationResult {
    pub code: Option<String>,
    pub required_actions: Vec<RequiredAction>,
    pub user_id: Uuid,
    pub token: Option<String>,
    pub credentials: Vec<String>,
}

pub trait AuthenticationService: Clone + Send + Sync + 'static {
    fn using_session_code(
        &self,
        realm_id: String,
        client_id: String,
        session_code: Uuid,
        username: String,
        password: String,
        base_url: String,
    ) -> impl Future<Output = Result<AuthenticationResult, AuthenticationError>> + Send;

    fn authenticate(
        &self,
        data: AuthenticateDto,
        base_url: String,
    ) -> impl Future<Output = Result<JwtToken, AuthenticationError>> + Send;
}
