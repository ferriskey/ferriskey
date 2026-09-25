use std::sync::Arc;

use chrono::Utc;
use ferriskey_domain::token_lifetime::TokenLifetimes;
use uuid::Uuid;

use crate::domain::authentication::ports::{OpenedSsoSession, SsoSessionPort};
use crate::domain::authentication::services::open_user_session;
use crate::domain::client::ports::ClientRepository;
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::realm::entities::RealmScope;
use crate::domain::realm::ports::RealmRepository;
use crate::domain::seawatch::SecurityEventRepository;
use crate::domain::session::ports::UserSessionRepository;

/// Opens SSO sessions the same way the authentication service does, for the
/// login steps that live outside it.
#[derive(Clone, Debug)]
pub struct SsoSessionAdapter<S, E, R, C>
where
    S: UserSessionRepository,
    E: SecurityEventRepository,
    R: RealmRepository,
    C: ClientRepository,
{
    session_repository: Arc<S>,
    security_event_repository: Arc<E>,
    realm_repository: Arc<R>,
    client_repository: Arc<C>,
}

impl<S, E, R, C> SsoSessionAdapter<S, E, R, C>
where
    S: UserSessionRepository,
    E: SecurityEventRepository,
    R: RealmRepository,
    C: ClientRepository,
{
    pub fn new(
        session_repository: Arc<S>,
        security_event_repository: Arc<E>,
        realm_repository: Arc<R>,
        client_repository: Arc<C>,
    ) -> Self {
        Self {
            session_repository,
            security_event_repository,
            realm_repository,
            client_repository,
        }
    }
}

impl<S, E, R, C> SsoSessionPort for SsoSessionAdapter<S, E, R, C>
where
    S: UserSessionRepository,
    E: SecurityEventRepository,
    R: RealmRepository,
    C: ClientRepository,
{
    async fn open_for_login(
        &self,
        scope: &RealmScope,
        user_id: Uuid,
        client_id: Uuid,
    ) -> Result<OpenedSsoSession, CoreError> {
        let realm_settings = self
            .realm_repository
            .get_realm_settings(scope.id())
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let client = self
            .client_repository
            .get_by_id(scope.id(), client_id)
            .await
            .map_err(|_| CoreError::InvalidClient)?
            .in_realm(scope)
            .map_err(|_| CoreError::InvalidClient)?
            .into_inner();

        let lifetimes = TokenLifetimes::resolve(&realm_settings, &client);

        let (session, cookie) = open_user_session(
            self.session_repository.as_ref(),
            self.security_event_repository.as_ref(),
            user_id,
            scope.id(),
            lifetimes.refresh_token,
        )
        .await?;

        Ok(OpenedSsoSession {
            session_id: session.id,
            cookie,
            max_age_secs: (session.expires_at - Utc::now()).num_seconds().max(0),
        })
    }
}
