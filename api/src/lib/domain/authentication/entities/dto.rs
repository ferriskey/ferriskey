use uuid::Uuid;

use super::grant_type::GrantType;

pub struct AuthenticateDto {
    pub realm_name: String,
    pub grant_type: GrantType,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub code: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub refresh_token: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreateAuthSessionDto {
    pub realm_id: Uuid,
    pub client_id: Uuid,
    pub redirect_uri: String,
    pub response_type: String,
    pub scope: String,
    pub state: Option<String>,
    pub nonce: Option<String>,
    pub user_id: Option<Uuid>,
}

impl CreateAuthSessionDto {
    pub fn new(realm_id: Uuid, client_id: Uuid, redirect_uri: String) -> Self {
        Self {
            realm_id,
            client_id,
            redirect_uri,
            response_type: "code".to_string(),
            scope: "openid".to_string(),
            state: None,
            nonce: None,
            user_id: None,
        }
    }

    pub fn with_oauth_params(
        mut self,
        response_type: String,
        scope: String,
        state: Option<String>,
        nonce: Option<String>,
    ) -> Self {
        self.response_type = response_type;
        self.scope = scope;
        self.state = state;
        self.nonce = nonce;
        self
    }

    pub fn with_auth_info(mut self, user_id: Option<Uuid>) -> Self {
        self.user_id = user_id;
        self
    }
}
