use crate::{
    application::common::services::{
        DefaultAuthSessionService, DefaultClientService, DefaultRealmService,
        DefaultRedirectUriService,
    },
    domain::{
        authentication::{
            entities::{AuthSession, AuthenticationError},
            ports::AuthSessionService,
            value_objects::CreateAuthSessionRequest,
        },
        client::ports::{OldClientService, RedirectUriService},
        realm::ports::RealmService,
    },
};

pub struct AuthUseCaseInput {
    pub realm_name: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub state: Option<String>,
    pub scope: Option<String>,
    pub response_type: String,
}
pub struct AuthUseCaseOutput {
    pub login_url: String,
    pub session: AuthSession,
}

#[derive(Clone)]
pub struct AuthUseCase {
    realm_service: DefaultRealmService,
    client_service: DefaultClientService,
    redirect_uri_service: DefaultRedirectUriService,
    auth_session_service: DefaultAuthSessionService,
}

impl AuthUseCase {
    pub fn new(
        realm_service: DefaultRealmService,
        client_service: DefaultClientService,
        redirect_uri_service: DefaultRedirectUriService,
        auth_session_service: DefaultAuthSessionService,
    ) -> Self {
        Self {
            realm_service,
            client_service,
            redirect_uri_service,
            auth_session_service,
        }
    }

    pub async fn execute(
        &self,
        input: AuthUseCaseInput,
    ) -> Result<AuthUseCaseOutput, AuthenticationError> {
        let realm = self
            .realm_service
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| AuthenticationError::InvalidRealm)?;

        let client = self
            .client_service
            .get_by_client_id(input.client_id.clone(), realm.id)
            .await
            .map_err(|_| AuthenticationError::InvalidClient)?;

        let redirect_uri = input.redirect_uri.clone();

        let client_redirect_uris = self
            .redirect_uri_service
            .get_enabled_by_client_id(client.id)
            .await
            .map_err(|_| AuthenticationError::InternalServerError)?;

        if !client_redirect_uris.iter().any(|uri| {
            if uri.value == redirect_uri {
                return true;
            }

            if let Ok(regex) = regex::Regex::new(&uri.value) {
                return regex.is_match(&redirect_uri);
            }

            false
        }) {
            return Err(AuthenticationError::InvalidClient);
        }

        if !client.enabled {
            return Err(AuthenticationError::InvalidClient);
        }

        let session = self
            .auth_session_service
            .create_session(CreateAuthSessionRequest {
                state: input.state.clone(),
                client_id: client.id,
                redirect_uri,
                scope: input.scope.unwrap_or_default(),
                nonce: None,
                response_type: input.response_type.clone(),
                realm_id: realm.id,
                user_id: None,
            })
            .await
            .map_err(|_| AuthenticationError::InternalServerError)?;

        let login_url = format!(
            "?client_id={}&redirect_uri={}&state={}",
            client.client_id,
            input.redirect_uri,
            input.state.unwrap_or_default()
        );

        Ok(AuthUseCaseOutput { login_url, session })
    }
}
