use std::sync::Arc;

use uuid::Uuid;

use crate::application::services::ApplicationAuthService;
use crate::domain::authentication::assurance::AcrRequest;
use crate::domain::authentication::services::sso_token_hash;
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::common::generate_random_token;
use crate::domain::device_session::ports::SessionResumptionPort;
use crate::domain::realm::entities::{RealmScope, Scoped};
use crate::domain::session::entities::UserSession;
use crate::domain::user::entities::User;

#[derive(Clone)]
pub struct SessionResumptionAdapter {
    auth_service: Arc<ApplicationAuthService>,
}

impl std::fmt::Debug for SessionResumptionAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionResumptionAdapter")
            .finish_non_exhaustive()
    }
}

impl SessionResumptionAdapter {
    pub fn new(auth_service: Arc<ApplicationAuthService>) -> Self {
        Self { auth_service }
    }
}

impl SessionResumptionPort for SessionResumptionAdapter {
    fn generate_secret(&self) -> String {
        generate_random_token()
    }

    fn hash_secret(&self, secret: &str) -> String {
        sso_token_hash(secret)
    }

    async fn resumable_session(
        &self,
        scope: &RealmScope,
        session: Scoped<UserSession>,
        max_age: Option<i64>,
        client_id: Uuid,
    ) -> Result<(Scoped<UserSession>, User), CoreError> {
        let (session, user, _) = self
            .auth_service
            .resumable_session(session, scope, max_age, &AcrRequest::default(), client_id)
            .await?;

        Ok((session, user))
    }
}
