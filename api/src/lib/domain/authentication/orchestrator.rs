use crate::domain::{
    authentication::{
        entities::error::AuthenticationError,
        service::{
            auth_session::DefaultAuthSessionService, authentication::DefaultAuthenticationService,
        },
        use_cases::{
            authenticate_use_case::{AuthenticateCommand, AuthenticateResult, AuthenticateUseCase},
            get_certs_use_case::GetCertsUseCase,
        },
    },
    jwt::{entities::jwt::JwkKey, services::jwt_service::DefaultJwtService},
    realm::services::realm_service::DefaultRealmService,
};

#[derive(Clone)]
pub struct AuthenticationOrchestrator {
    authenticate_use_case: AuthenticateUseCase,
    get_certs_use_case: GetCertsUseCase,
}

impl AuthenticationOrchestrator {
    pub fn new(
        realm_service: DefaultRealmService,
        auth_session_service: DefaultAuthSessionService,
        jwt_service: DefaultJwtService,
        authentication_service: DefaultAuthenticationService,
    ) -> Self {
        let authenticate_use_case = AuthenticateUseCase::new(
            realm_service.clone(),
            auth_session_service,
            jwt_service.clone(),
            authentication_service,
        );

        let get_certs_use_case = GetCertsUseCase::new(realm_service, jwt_service);

        Self {
            authenticate_use_case,
            get_certs_use_case,
        }
    }

    pub async fn authenticate(
        &self,
        command: AuthenticateCommand,
    ) -> Result<AuthenticateResult, AuthenticationError> {
        self.authenticate_use_case.execute(command).await
    }

    pub async fn get_certs(&self, realm_name: String) -> Result<Vec<JwkKey>, AuthenticationError> {
        self.get_certs_use_case
            .execute(realm_name)
            .await
            .map_err(|_| AuthenticationError::InternalServerError)
    }
}
