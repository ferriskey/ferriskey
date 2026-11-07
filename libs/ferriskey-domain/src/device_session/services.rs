use std::sync::Arc;

use chrono::Utc;
use tracing::warn;
use uuid::Uuid;

use crate::common::app_errors::CoreError;
use crate::device_session::entities::{
    AccountEntry, DeviceSession, DeviceSessionError, SelectedAccount, display_name, mask_email,
};
use crate::device_session::ports::{
    AccountChooserService, DeviceSessionRepository, SessionResumptionPort,
};
use crate::realm::ports::RealmRepository;
use crate::realm::scope::{RealmScope, Scoped, Unscoped, UnscopedOption};
use crate::session::entities::{SessionError, UserSession};
use crate::session::ports::UserSessionRepository;
use crate::user::ports::UserRepository;

#[derive(Clone, Debug)]
pub struct AccountChooserServiceImpl<D, S, U, R, G>
where
    D: DeviceSessionRepository,
    S: UserSessionRepository,
    U: UserRepository,
    R: RealmRepository,
    G: SessionResumptionPort,
{
    device_repository: Arc<D>,
    session_repository: Arc<S>,
    user_repository: Arc<U>,
    realm_repository: Arc<R>,
    resumption: Arc<G>,
}

impl<D, S, U, R, G> AccountChooserServiceImpl<D, S, U, R, G>
where
    D: DeviceSessionRepository,
    S: UserSessionRepository,
    U: UserRepository,
    R: RealmRepository,
    G: SessionResumptionPort,
{
    pub fn new(
        device_repository: Arc<D>,
        session_repository: Arc<S>,
        user_repository: Arc<U>,
        realm_repository: Arc<R>,
        resumption: Arc<G>,
    ) -> Self {
        Self {
            device_repository,
            session_repository,
            user_repository,
            realm_repository,
            resumption,
        }
    }

    async fn resolve_device(
        &self,
        device_token: Option<&str>,
        scope: &RealmScope,
    ) -> Result<Option<Scoped<DeviceSession>>, CoreError> {
        let Some(token) = device_token.filter(|token| !token.is_empty()) else {
            return Ok(None);
        };

        let device = self
            .device_repository
            .find_by_token_hash(&self.resumption.hash_secret(token))
            .await
            .map_err(|e| {
                warn!(error = ?e, "Failed to load a device session");
                CoreError::InternalServerError
            })?;

        let Some(device) = device else {
            return Ok(None);
        };

        let Ok(device) = device.in_realm(scope) else {
            return Ok(None);
        };

        Ok((!device.get().is_expired(Utc::now())).then_some(device))
    }

    async fn sessions_of_device(
        &self,
        device: &Scoped<DeviceSession>,
        scope: &RealmScope,
    ) -> Result<Vec<Scoped<UserSession>>, CoreError> {
        let sessions = self
            .session_repository
            .find_all_by_device(device.get().id, device.get().realm_id)
            .await
            .map_err(|e| {
                warn!(error = ?e, "Failed to list the sessions of a device");
                CoreError::InternalServerError
            })?;

        Ok(sessions
            .into_iter()
            .filter_map(|session| Unscoped::new(session).in_realm(scope).ok())
            .collect())
    }

    async fn attached_session(
        &self,
        device_token: Option<&str>,
        scope: &RealmScope,
        session_id: Uuid,
    ) -> Result<Scoped<UserSession>, CoreError> {
        let device = self
            .resolve_device(device_token, scope)
            .await?
            .ok_or(CoreError::DeviceSessionNotFound)?;

        self.sessions_of_device(&device, scope)
            .await?
            .into_iter()
            .find(|session| session.get().id == session_id)
            .ok_or(CoreError::DeviceSessionNotFound)
    }
}

fn supersedes(candidate: &AccountEntry, kept: &AccountEntry) -> bool {
    (candidate.is_active, candidate.last_seen_at) > (kept.is_active, kept.last_seen_at)
}

fn device_failure(error: DeviceSessionError) -> CoreError {
    warn!(error = ?error, "A device session operation failed");
    CoreError::InternalServerError
}

fn session_failure(error: SessionError) -> CoreError {
    match error {
        SessionError::NotFound => CoreError::SessionNotFound,
        other => {
            warn!(error = ?other, "A session operation failed");
            CoreError::InternalServerError
        }
    }
}

impl<D, S, U, R, G> AccountChooserService for AccountChooserServiceImpl<D, S, U, R, G>
where
    D: DeviceSessionRepository,
    S: UserSessionRepository,
    U: UserRepository,
    R: RealmRepository,
    G: SessionResumptionPort,
{
    async fn attach_session_to_device(
        &self,
        device_token: Option<&str>,
        scope: &RealmScope,
        user_session_id: Uuid,
    ) -> Result<(DeviceSession, Option<String>), CoreError> {
        let session = self
            .session_repository
            .find_by_id(user_session_id)
            .await
            .map_err(session_failure)?
            .in_realm(scope)?
            .ok_or(CoreError::SessionNotFound)?;

        let now = Utc::now();
        let session_expires_at = session.get().expires_at;

        let (device, new_token) = match self.resolve_device(device_token, scope).await? {
            Some(existing) => {
                let expires_at = DeviceSession::extended_expiry(
                    Some(existing.get().expires_at),
                    session_expires_at,
                    now,
                );
                self.device_repository
                    .touch_last_seen(&existing, expires_at)
                    .await
                    .map_err(device_failure)?;

                let device = DeviceSession {
                    last_seen_at: now,
                    expires_at,
                    ..existing.into_inner()
                };
                (device, None)
            }
            None => {
                let token = self.resumption.generate_secret();
                let device = DeviceSession::new(
                    Uuid::from(scope.id()),
                    self.resumption.hash_secret(&token),
                    session_expires_at,
                    now,
                );
                self.device_repository
                    .create(&device)
                    .await
                    .map_err(device_failure)?;

                (device, Some(token))
            }
        };

        self.session_repository
            .attach_device(&session, device.id)
            .await
            .map_err(session_failure)?;

        if let Err(e) = self
            .device_repository
            .delete_expired(device.realm_id, now)
            .await
        {
            warn!(error = ?e, "Failed to purge the expired device sessions");
        }

        Ok((device, new_token))
    }

    async fn list_entries(
        &self,
        device_token: Option<&str>,
        scope: &RealmScope,
        active_sso_token: Option<&str>,
        max_age: Option<i64>,
        client_id: Uuid,
    ) -> Result<Vec<AccountEntry>, CoreError> {
        let settings = self
            .realm_repository
            .get_realm_settings(scope.id())
            .await?
            .filter(|settings| settings.account_chooser_enabled);

        let Some(settings) = settings else {
            return Ok(Vec::new());
        };

        let Some(device) = self.resolve_device(device_token, scope).await? else {
            return Ok(Vec::new());
        };

        let active_hash = active_sso_token.map(|token| self.resumption.hash_secret(token));

        let mut entries: Vec<(Uuid, AccountEntry)> = Vec::new();

        for session in self.sessions_of_device(&device, scope).await? {
            if session.get().is_expired() || session.get().sso_token_hash.is_none() {
                continue;
            }

            let Ok(user) = self.user_repository.get_by_id(session.get().user_id).await else {
                continue;
            };
            let Ok(user) = user.in_realm(scope) else {
                continue;
            };
            let user = user.into_inner();

            if !user.enabled {
                continue;
            }

            let is_active = active_hash
                .as_deref()
                .is_some_and(|hash| session.get().sso_token_hash.as_deref() == Some(hash));

            let candidate = session.get().clone();

            let requires_reauthentication = match self
                .resumption
                .resumable_session(scope, session, max_age, client_id)
                .await
            {
                Ok((_, user)) => user.is_locked(Utc::now()),
                Err(
                    CoreError::SessionExpired
                    | CoreError::SessionNotFound
                    | CoreError::UserDisabled,
                ) => continue,
                Err(e @ (CoreError::InternalServerError | CoreError::InvalidClient)) => {
                    return Err(e);
                }
                Err(_) => true,
            };

            let entry = AccountEntry {
                session_id: candidate.id,
                display_name: display_name(&user),
                masked_email: settings
                    .account_chooser_show_email
                    .then(|| user.email.as_deref().map(mask_email))
                    .flatten()
                    .filter(|masked| !masked.is_empty()),
                state: candidate.get_state(),
                last_seen_at: candidate.last_seen_at,
                is_active,
                requires_reauthentication,
            };

            match entries.iter_mut().find(|(id, _)| *id == candidate.user_id) {
                Some((_, kept)) if supersedes(&entry, kept) => *kept = entry,
                Some(_) => {}
                None => entries.push((candidate.user_id, entry)),
            }
        }

        let mut entries: Vec<AccountEntry> = entries.into_iter().map(|(_, entry)| entry).collect();
        entries.sort_by_key(|entry| std::cmp::Reverse(entry.last_seen_at));

        Ok(entries)
    }

    async fn select_entry(
        &self,
        device_token: Option<&str>,
        scope: &RealmScope,
        session_id: Uuid,
        max_age: Option<i64>,
        client_id: Uuid,
    ) -> Result<SelectedAccount, CoreError> {
        let session = self
            .attached_session(device_token, scope, session_id)
            .await?;

        if session.get().sso_token_hash.is_none() {
            return Err(CoreError::SessionNotFound);
        }

        let (session, user) = self
            .resumption
            .resumable_session(scope, session, max_age, client_id)
            .await?;

        let new_sso_secret = self.resumption.generate_secret();

        self.session_repository
            .set_sso_token_hash(&session, &self.resumption.hash_secret(&new_sso_secret))
            .await
            .map_err(session_failure)?;

        Ok(SelectedAccount {
            user_id: user.id,
            session_id: session.get().id,
            new_sso_secret,
            cookie_max_age: session.get().cookie_max_age(),
        })
    }

    async fn detach_entry(
        &self,
        device_token: Option<&str>,
        scope: &RealmScope,
        session_id: Uuid,
    ) -> Result<(), CoreError> {
        let session = self
            .attached_session(device_token, scope, session_id)
            .await?;

        self.session_repository
            .clear_sso_token_hash(&session)
            .await
            .map_err(session_failure)?;

        self.session_repository
            .detach_device(&session)
            .await
            .map_err(session_failure)
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;
    use crate::device_session::ports::{MockDeviceSessionRepository, MockSessionResumptionPort};
    use crate::realm::ports::MockRealmRepository;
    use crate::realm::{Realm, RealmId, RealmSetting};
    use crate::session::ports::MockUserSessionRepository;
    use crate::user::entities::User;
    use crate::user::ports::MockUserRepository;

    type Service = AccountChooserServiceImpl<
        MockDeviceSessionRepository,
        MockUserSessionRepository,
        MockUserRepository,
        MockRealmRepository,
        MockSessionResumptionPort,
    >;

    const TOKEN: &str = "device-token";

    fn hashed(secret: &str) -> String {
        format!("h:{secret}")
    }

    fn scope() -> RealmScope {
        let mut realm = Realm::new("acme".to_owned());
        realm.id = RealmId::new(Uuid::new_v4());
        RealmScope::from_realm(realm)
    }

    fn settings(scope: &RealmScope, enabled: bool, show_email: bool) -> RealmSetting {
        let mut settings = RealmSetting::new(scope.id(), None);
        settings.account_chooser_enabled = enabled;
        settings.account_chooser_show_email = show_email;
        settings
    }

    fn device(scope: &RealmScope) -> DeviceSession {
        let now = Utc::now();
        DeviceSession::new(
            Uuid::from(scope.id()),
            hashed(TOKEN),
            now + Duration::hours(1),
            now,
        )
    }

    fn session(scope: &RealmScope, user_id: Uuid) -> UserSession {
        let mut session = UserSession::new(
            user_id,
            Uuid::from(scope.id()),
            None,
            None,
            Duration::hours(1),
            None,
        );
        session.sso_token_hash = Some(hashed(&Uuid::new_v4().to_string()));
        session
    }

    fn user(scope: &RealmScope) -> User {
        user_in(scope.id())
    }

    fn resumption() -> MockSessionResumptionPort {
        let mut port = MockSessionResumptionPort::new();
        port.expect_hash_secret().returning(hashed);
        port.expect_generate_secret()
            .returning(|| "fresh-secret".to_owned());
        port
    }

    fn resumption_passing() -> MockSessionResumptionPort {
        let mut port = resumption();
        port.expect_resumable_session()
            .returning(|scope, session, _, _| {
                let realm = scope.id();
                let mut user = user_in(realm);
                user.id = session.get().user_id;
                Box::pin(async move { Ok((session, user)) })
            });
        port
    }

    fn user_in(realm: RealmId) -> User {
        let now = Utc::now();
        User {
            id: Uuid::new_v4(),
            realm_id: realm,
            client_id: None,
            username: "alice".to_owned(),
            firstname: Some("Alice".to_owned()),
            lastname: Some("Doe".to_owned()),
            email: Some("alice@example.com".to_owned()),
            email_verified: true,
            enabled: true,
            roles: None,
            realm: None,
            required_actions: Vec::new(),
            created_at: now,
            updated_at: now,
            failed_login_attempts: 0,
            locked_until: None,
            locale: None,
            federation_provider_id: None,
        }
    }

    fn realms(scope: &RealmScope, enabled: bool, show_email: bool) -> MockRealmRepository {
        let settings = settings(scope, enabled, show_email);
        let mut realms = MockRealmRepository::new();
        realms.expect_get_realm_settings().returning(move |_| {
            let settings = settings.clone();
            Box::pin(async move { Ok(Some(settings)) })
        });
        realms
    }

    fn devices_resolving(device: DeviceSession) -> MockDeviceSessionRepository {
        let mut devices = MockDeviceSessionRepository::new();
        devices.expect_find_by_token_hash().returning(move |hash| {
            let found = (hash == device.device_token_hash).then(|| Unscoped::new(device.clone()));
            Box::pin(async move { Ok(found) })
        });
        devices
    }

    fn users_returning(users: Vec<User>) -> MockUserRepository {
        let mut repository = MockUserRepository::new();
        repository.expect_get_by_id().returning(move |id| {
            let found = users.iter().find(|u| u.id == id).cloned();
            Box::pin(async move { found.map(Unscoped::new).ok_or(CoreError::NotFound) })
        });
        repository
    }

    fn sessions_of(
        mut sessions: MockUserSessionRepository,
        list: Vec<UserSession>,
    ) -> MockUserSessionRepository {
        sessions.expect_find_all_by_device().returning(move |_, _| {
            let list = list.clone();
            Box::pin(async move { Ok(list) })
        });
        sessions
    }

    fn service(
        devices: MockDeviceSessionRepository,
        sessions: MockUserSessionRepository,
        users: MockUserRepository,
        realms: MockRealmRepository,
        resumption: MockSessionResumptionPort,
    ) -> Service {
        AccountChooserServiceImpl::new(
            Arc::new(devices),
            Arc::new(sessions),
            Arc::new(users),
            Arc::new(realms),
            Arc::new(resumption),
        )
    }

    async fn list(service: &Service, scope: &RealmScope, token: Option<&str>) -> Vec<AccountEntry> {
        service
            .list_entries(token, scope, None, None, Uuid::new_v4())
            .await
            .expect("listing must not fail")
    }

    #[tokio::test]
    async fn list_keeps_a_live_entry_of_an_enabled_user() {
        let scope = scope();
        let user = user(&scope);
        let live = session(&scope, user.id);
        let service = service(
            devices_resolving(device(&scope)),
            sessions_of(MockUserSessionRepository::new(), vec![live.clone()]),
            users_returning(vec![user]),
            realms(&scope, true, false),
            resumption_passing(),
        );

        let entries = list(&service, &scope, Some(TOKEN)).await;

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].session_id, live.id);
        assert_eq!(entries[0].display_name, "Alice Doe");
        assert_eq!(entries[0].masked_email, None);
        assert!(!entries[0].is_active);
        assert!(!entries[0].requires_reauthentication);
    }

    #[tokio::test]
    async fn list_drops_expired_revoked_disabled_and_foreign_realm_entries() {
        let scope = scope();
        let user = user(&scope);
        let mut disabled = user_in(scope.id());
        disabled.enabled = false;
        let foreign_user = user_in(RealmId::new(Uuid::new_v4()));

        let live = session(&scope, user.id);
        let mut expired = session(&scope, user.id);
        expired.expires_at = Utc::now() - Duration::minutes(1);
        let mut revoked = session(&scope, user.id);
        revoked.sso_token_hash = None;
        let of_disabled_user = session(&scope, disabled.id);
        let of_foreign_user = session(&scope, foreign_user.id);
        let mut of_other_realm = session(&scope, user.id);
        of_other_realm.realm_id = Uuid::new_v4();

        let service = service(
            devices_resolving(device(&scope)),
            sessions_of(
                MockUserSessionRepository::new(),
                vec![
                    expired,
                    revoked,
                    of_disabled_user,
                    of_foreign_user,
                    of_other_realm,
                    live.clone(),
                ],
            ),
            users_returning(vec![user, disabled, foreign_user]),
            realms(&scope, true, false),
            resumption_passing(),
        );

        let entries = list(&service, &scope, Some(TOKEN)).await;

        assert_eq!(
            entries.iter().map(|e| e.session_id).collect::<Vec<_>>(),
            vec![live.id]
        );
    }

    #[tokio::test]
    async fn list_of_an_unknown_or_missing_device_token_is_empty() {
        let scope = scope();
        let service = service(
            devices_resolving(device(&scope)),
            MockUserSessionRepository::new(),
            MockUserRepository::new(),
            realms(&scope, true, false),
            resumption(),
        );

        assert!(list(&service, &scope, Some("forged")).await.is_empty());
        assert!(list(&service, &scope, Some("")).await.is_empty());
        assert!(list(&service, &scope, None).await.is_empty());
    }

    #[tokio::test]
    async fn list_of_a_device_of_another_realm_is_empty() {
        let scope = scope();
        let mut foreign = device(&scope);
        foreign.realm_id = Uuid::new_v4();
        let service = service(
            devices_resolving(foreign),
            MockUserSessionRepository::new(),
            MockUserRepository::new(),
            realms(&scope, true, false),
            resumption(),
        );

        assert!(list(&service, &scope, Some(TOKEN)).await.is_empty());
    }

    #[tokio::test]
    async fn list_of_an_expired_device_is_empty() {
        let scope = scope();
        let mut expired = device(&scope);
        expired.expires_at = Utc::now() - Duration::minutes(1);
        let service = service(
            devices_resolving(expired),
            MockUserSessionRepository::new(),
            MockUserRepository::new(),
            realms(&scope, true, false),
            resumption(),
        );

        assert!(list(&service, &scope, Some(TOKEN)).await.is_empty());
    }

    #[tokio::test]
    async fn list_is_empty_when_the_chooser_is_disabled() {
        let scope = scope();
        let service = service(
            MockDeviceSessionRepository::new(),
            MockUserSessionRepository::new(),
            MockUserRepository::new(),
            realms(&scope, false, true),
            resumption(),
        );

        assert!(list(&service, &scope, Some(TOKEN)).await.is_empty());
    }

    #[tokio::test]
    async fn list_masks_the_email_only_when_the_realm_shows_it() {
        let scope = scope();
        let user = user(&scope);
        let live = session(&scope, user.id);
        let service = service(
            devices_resolving(device(&scope)),
            sessions_of(MockUserSessionRepository::new(), vec![live]),
            users_returning(vec![user]),
            realms(&scope, true, true),
            resumption_passing(),
        );

        let entries = list(&service, &scope, Some(TOKEN)).await;

        assert_eq!(
            entries[0].masked_email.as_deref(),
            Some("a••••@example.com")
        );
    }

    #[tokio::test]
    async fn list_marks_the_entry_the_active_cookie_resolves_to() {
        let scope = scope();
        let user = user(&scope);
        let mut active = session(&scope, user.id);
        active.sso_token_hash = Some(hashed("active-secret"));
        let other = session(&scope, user.id);
        let service = service(
            devices_resolving(device(&scope)),
            sessions_of(
                MockUserSessionRepository::new(),
                vec![active.clone(), other],
            ),
            users_returning(vec![user]),
            realms(&scope, true, false),
            resumption_passing(),
        );

        let entries = service
            .list_entries(
                Some(TOKEN),
                &scope,
                Some("active-secret"),
                None,
                Uuid::new_v4(),
            )
            .await
            .expect("listing must not fail");

        let flagged: Vec<Uuid> = entries
            .iter()
            .filter(|e| e.is_active)
            .map(|e| e.session_id)
            .collect();
        assert_eq!(flagged, vec![active.id]);
    }

    #[tokio::test]
    async fn list_keeps_the_most_recent_entry_of_a_user() {
        let scope = scope();
        let user = user(&scope);
        let now = Utc::now();
        let mut older = session(&scope, user.id);
        older.last_seen_at = Some(now - Duration::minutes(10));
        let mut newer = session(&scope, user.id);
        newer.last_seen_at = Some(now);
        let service = service(
            devices_resolving(device(&scope)),
            sessions_of(MockUserSessionRepository::new(), vec![older, newer.clone()]),
            users_returning(vec![user]),
            realms(&scope, true, false),
            resumption_passing(),
        );

        let entries = list(&service, &scope, Some(TOKEN)).await;

        let ids: Vec<Uuid> = entries.iter().map(|e| e.session_id).collect();
        assert_eq!(ids, vec![newer.id]);
    }

    #[tokio::test]
    async fn list_keeps_the_active_entry_of_a_user_even_when_older() {
        let scope = scope();
        let user = user(&scope);
        let other = user_in(scope.id());
        let now = Utc::now();
        let mut active = session(&scope, user.id);
        active.sso_token_hash = Some(hashed("active-secret"));
        active.last_seen_at = Some(now - Duration::minutes(10));
        let mut newer = session(&scope, user.id);
        newer.last_seen_at = Some(now);
        let bob = session(&scope, other.id);
        let service = service(
            devices_resolving(device(&scope)),
            sessions_of(
                MockUserSessionRepository::new(),
                vec![newer, active.clone(), bob.clone()],
            ),
            users_returning(vec![user, other]),
            realms(&scope, true, false),
            resumption_passing(),
        );

        let entries = service
            .list_entries(
                Some(TOKEN),
                &scope,
                Some("active-secret"),
                None,
                Uuid::new_v4(),
            )
            .await
            .expect("listing must not fail");

        let mut ids: Vec<Uuid> = entries.iter().map(|e| e.session_id).collect();
        ids.sort();
        let mut expected = vec![active.id, bob.id];
        expected.sort();
        assert_eq!(ids, expected);
        assert!(
            entries
                .iter()
                .any(|e| e.is_active && e.session_id == active.id)
        );
    }

    async fn requires_reauthentication_when(
        guard: Result<(), CoreError>,
        lock_the_user: bool,
    ) -> bool {
        let scope = scope();
        let mut user = user(&scope);
        if lock_the_user {
            user.locked_until = Some(Utc::now() + Duration::minutes(10));
        }
        let live = session(&scope, user.id);

        let mut port = resumption();
        let returned = user.clone();
        port.expect_resumable_session()
            .returning(move |_, session, _, _| {
                let user = returned.clone();
                let guard = guard.clone();
                Box::pin(async move { guard.map(|()| (session, user)) })
            });

        let service = service(
            devices_resolving(device(&scope)),
            sessions_of(MockUserSessionRepository::new(), vec![live]),
            users_returning(vec![user]),
            realms(&scope, true, false),
            port,
        );

        let entries = list(&service, &scope, Some(TOKEN)).await;
        assert_eq!(entries.len(), 1, "the account must stay listed");
        entries[0].requires_reauthentication
    }

    #[tokio::test]
    async fn list_flags_an_entry_whose_max_age_is_exceeded() {
        assert!(
            requires_reauthentication_when(Err(CoreError::ReauthenticationRequired), false).await
        );
    }

    #[tokio::test]
    async fn list_flags_an_entry_that_owes_a_required_action() {
        let owed = CoreError::Forbidden("an authentication step is still due".to_owned());
        assert!(requires_reauthentication_when(Err(owed), false).await);
    }

    #[tokio::test]
    async fn list_flags_an_entry_of_a_locked_user() {
        assert!(requires_reauthentication_when(Ok(()), true).await);
    }

    #[tokio::test]
    async fn list_does_not_flag_a_resumable_entry() {
        assert!(!requires_reauthentication_when(Ok(()), false).await);
    }

    #[tokio::test]
    async fn list_fails_when_the_guard_cannot_judge_an_entry() {
        let scope = scope();
        let user = user(&scope);
        let live = session(&scope, user.id);
        let mut port = resumption();
        port.expect_resumable_session()
            .returning(|_, _, _, _| Box::pin(async { Err(CoreError::InvalidClient) }));
        let service = service(
            devices_resolving(device(&scope)),
            sessions_of(MockUserSessionRepository::new(), vec![live]),
            users_returning(vec![user]),
            realms(&scope, true, false),
            port,
        );

        let result = service
            .list_entries(Some(TOKEN), &scope, None, None, Uuid::new_v4())
            .await;

        assert!(matches!(result, Err(CoreError::InvalidClient)));
    }

    fn selecting(
        scope: &RealmScope,
        target: UserSession,
        guard: Result<(), CoreError>,
        sessions: MockUserSessionRepository,
    ) -> Service {
        let user = user(scope);
        let mut port = resumption();
        port.expect_resumable_session()
            .returning(move |_, session, _, _| {
                let mut user = user.clone();
                user.id = session.get().user_id;
                let guard = guard.clone();
                Box::pin(async move { guard.map(|()| (session, user)) })
            });

        service(
            devices_resolving(device(scope)),
            sessions_of(sessions, vec![target]),
            MockUserRepository::new(),
            MockRealmRepository::new(),
            port,
        )
    }

    #[tokio::test]
    async fn select_mints_a_fresh_secret_through_set_sso_token_hash_only() {
        let scope = scope();
        let mut target = session(&scope, Uuid::new_v4());
        target.persistent = true;
        let before = target.clone();
        let target_id = target.id;

        let mut sessions = MockUserSessionRepository::new();
        sessions
            .expect_set_sso_token_hash()
            .times(1)
            .withf(move |session, hash| session.get().id == target_id && hash == "h:fresh-secret")
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let service = selecting(&scope, target.clone(), Ok(()), sessions);

        let selected = service
            .select_entry(Some(TOKEN), &scope, target_id, None, Uuid::new_v4())
            .await
            .expect("a resumable entry must be selectable");

        assert_eq!(selected.session_id, target_id);
        assert_eq!(selected.user_id, before.user_id);
        assert_eq!(selected.new_sso_secret, "fresh-secret");
        assert!(selected.cookie_max_age.is_some_and(|age| age > 0));
        assert_eq!(target.authenticated_at, before.authenticated_at);
        assert_eq!(target.expires_at, before.expires_at);
        assert_eq!(target.persistent, before.persistent);
    }

    #[tokio::test]
    async fn select_of_a_non_persistent_session_sets_a_browser_session_cookie() {
        let scope = scope();
        let target = session(&scope, Uuid::new_v4());
        let target_id = target.id;
        let mut sessions = MockUserSessionRepository::new();
        sessions
            .expect_set_sso_token_hash()
            .returning(|_, _| Box::pin(async { Ok(()) }));
        let service = selecting(&scope, target, Ok(()), sessions);

        let selected = service
            .select_entry(Some(TOKEN), &scope, target_id, None, Uuid::new_v4())
            .await
            .expect("a resumable entry must be selectable");

        assert_eq!(selected.cookie_max_age, None);
    }

    async fn selection_refused_with(reason: CoreError) -> CoreError {
        let scope = scope();
        let target = session(&scope, Uuid::new_v4());
        let target_id = target.id;
        let mut sessions = MockUserSessionRepository::new();
        sessions.expect_set_sso_token_hash().never();
        let service = selecting(&scope, target, Err(reason), sessions);

        service
            .select_entry(Some(TOKEN), &scope, target_id, Some(60), Uuid::new_v4())
            .await
            .expect_err("a refused selection must not succeed")
    }

    #[tokio::test]
    async fn select_is_refused_for_an_expired_session() {
        assert!(matches!(
            selection_refused_with(CoreError::SessionExpired).await,
            CoreError::SessionExpired
        ));
    }

    #[tokio::test]
    async fn select_is_refused_for_a_disabled_user() {
        assert!(matches!(
            selection_refused_with(CoreError::UserDisabled).await,
            CoreError::UserDisabled
        ));
    }

    #[tokio::test]
    async fn select_is_refused_while_a_required_action_is_owed() {
        let owed = CoreError::Forbidden("an authentication step is still due".to_owned());
        assert!(matches!(
            selection_refused_with(owed).await,
            CoreError::Forbidden(_)
        ));
    }

    #[tokio::test]
    async fn select_is_refused_when_max_age_is_exceeded() {
        assert!(matches!(
            selection_refused_with(CoreError::ReauthenticationRequired).await,
            CoreError::ReauthenticationRequired
        ));
    }

    #[tokio::test]
    async fn select_passes_max_age_and_client_to_the_guard() {
        let scope = scope();
        let target = session(&scope, Uuid::new_v4());
        let target_id = target.id;
        let client_id = Uuid::new_v4();
        let mut port = resumption();
        port.expect_resumable_session()
            .times(1)
            .withf(move |_, session, max_age, client| {
                session.get().id == target_id && *max_age == Some(30) && *client == client_id
            })
            .returning(|_, _, _, _| Box::pin(async { Err(CoreError::ReauthenticationRequired) }));
        let service = service(
            devices_resolving(device(&scope)),
            sessions_of(MockUserSessionRepository::new(), vec![target]),
            MockUserRepository::new(),
            MockRealmRepository::new(),
            port,
        );

        let result = service
            .select_entry(Some(TOKEN), &scope, target_id, Some(30), client_id)
            .await;

        assert!(matches!(result, Err(CoreError::ReauthenticationRequired)));
    }

    #[tokio::test]
    async fn select_of_a_session_not_on_the_device_is_not_found() {
        let scope = scope();
        let service = selecting(
            &scope,
            session(&scope, Uuid::new_v4()),
            Ok(()),
            MockUserSessionRepository::new(),
        );

        let result = service
            .select_entry(Some(TOKEN), &scope, Uuid::new_v4(), None, Uuid::new_v4())
            .await;

        assert!(matches!(result, Err(CoreError::DeviceSessionNotFound)));
    }

    #[tokio::test]
    async fn select_with_an_unknown_device_is_not_found() {
        let scope = scope();
        let target = session(&scope, Uuid::new_v4());
        let target_id = target.id;
        let service = selecting(&scope, target, Ok(()), MockUserSessionRepository::new());

        let forged = service
            .select_entry(Some("forged"), &scope, target_id, None, Uuid::new_v4())
            .await;
        let absent = service
            .select_entry(None, &scope, target_id, None, Uuid::new_v4())
            .await;

        assert!(matches!(forged, Err(CoreError::DeviceSessionNotFound)));
        assert!(matches!(absent, Err(CoreError::DeviceSessionNotFound)));
    }

    #[tokio::test]
    async fn select_of_a_revoked_session_is_refused_before_the_guard() {
        let scope = scope();
        let mut target = session(&scope, Uuid::new_v4());
        target.sso_token_hash = None;
        let target_id = target.id;
        let mut sessions = MockUserSessionRepository::new();
        sessions.expect_set_sso_token_hash().never();
        let service = service(
            devices_resolving(device(&scope)),
            sessions_of(sessions, vec![target]),
            MockUserRepository::new(),
            MockRealmRepository::new(),
            resumption(),
        );

        let result = service
            .select_entry(Some(TOKEN), &scope, target_id, None, Uuid::new_v4())
            .await;

        assert!(matches!(result, Err(CoreError::SessionNotFound)));
    }

    #[tokio::test]
    async fn detach_clears_the_secret_then_detaches_without_deleting() {
        let scope = scope();
        let target = session(&scope, Uuid::new_v4());
        let target_id = target.id;
        let order = Arc::new(std::sync::Mutex::new(Vec::new()));

        let mut sessions = MockUserSessionRepository::new();
        let log = order.clone();
        sessions
            .expect_clear_sso_token_hash()
            .times(1)
            .returning(move |_| {
                log.lock().expect("log").push("clear");
                Box::pin(async { Ok(()) })
            });
        let log = order.clone();
        sessions
            .expect_detach_device()
            .times(1)
            .returning(move |_| {
                log.lock().expect("log").push("detach");
                Box::pin(async { Ok(()) })
            });
        sessions.expect_delete().never();

        let service = service(
            devices_resolving(device(&scope)),
            sessions_of(sessions, vec![target]),
            MockUserRepository::new(),
            MockRealmRepository::new(),
            resumption(),
        );

        service
            .detach_entry(Some(TOKEN), &scope, target_id)
            .await
            .expect("detaching an attached entry must succeed");

        assert_eq!(*order.lock().expect("log"), vec!["clear", "detach"]);
    }

    #[tokio::test]
    async fn detach_of_a_session_not_on_the_device_changes_nothing() {
        let scope = scope();
        let mut sessions = MockUserSessionRepository::new();
        sessions.expect_clear_sso_token_hash().never();
        sessions.expect_detach_device().never();
        let service = service(
            devices_resolving(device(&scope)),
            sessions_of(sessions, vec![session(&scope, Uuid::new_v4())]),
            MockUserRepository::new(),
            MockRealmRepository::new(),
            resumption(),
        );

        let result = service
            .detach_entry(Some(TOKEN), &scope, Uuid::new_v4())
            .await;

        assert!(matches!(result, Err(CoreError::DeviceSessionNotFound)));
    }

    fn sessions_finding(target: UserSession) -> MockUserSessionRepository {
        let mut sessions = MockUserSessionRepository::new();
        sessions.expect_find_by_id().returning(move |_| {
            let found = Unscoped::new(target.clone());
            Box::pin(async move { Ok(Some(found)) })
        });
        sessions
    }

    #[tokio::test]
    async fn attach_creates_a_device_when_the_browser_has_none() {
        let scope = scope();
        let target = session(&scope, Uuid::new_v4());
        let target_id = target.id;
        let target_expiry = target.expires_at;

        let mut devices = MockDeviceSessionRepository::new();
        devices
            .expect_create()
            .times(1)
            .withf(move |device| {
                device.device_token_hash == "h:fresh-secret" && device.expires_at == target_expiry
            })
            .returning(|_| Box::pin(async { Ok(()) }));
        devices
            .expect_delete_expired()
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(0) }));

        let mut sessions = sessions_finding(target);
        sessions
            .expect_attach_device()
            .times(1)
            .withf(move |session, _| session.get().id == target_id)
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let service = service(
            devices,
            sessions,
            MockUserRepository::new(),
            MockRealmRepository::new(),
            resumption(),
        );

        let (device, token) = service
            .attach_session_to_device(None, &scope, target_id)
            .await
            .expect("attaching must succeed");

        assert_eq!(token.as_deref(), Some("fresh-secret"));
        assert_eq!(device.device_token_hash, "h:fresh-secret");
        assert_eq!(Uuid::from(scope.id()), device.realm_id);
    }

    #[tokio::test]
    async fn attach_reuses_the_device_and_extends_its_lifetime() {
        let scope = scope();
        let mut target = session(&scope, Uuid::new_v4());
        target.expires_at = Utc::now() + Duration::days(3);
        let target_id = target.id;
        let target_expiry = target.expires_at;
        let existing = device(&scope);
        let existing_id = existing.id;

        let mut devices = devices_resolving(existing);
        devices
            .expect_touch_last_seen()
            .times(1)
            .withf(move |device, expires_at| {
                device.get().id == existing_id && *expires_at == target_expiry
            })
            .returning(|_, _| Box::pin(async { Ok(()) }));
        devices.expect_create().never();
        devices
            .expect_delete_expired()
            .returning(|_, _| Box::pin(async { Ok(0) }));

        let mut sessions = sessions_finding(target);
        sessions
            .expect_attach_device()
            .times(1)
            .withf(move |_, device_id| *device_id == existing_id)
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let service = service(
            devices,
            sessions,
            MockUserRepository::new(),
            MockRealmRepository::new(),
            resumption(),
        );

        let (device, token) = service
            .attach_session_to_device(Some(TOKEN), &scope, target_id)
            .await
            .expect("attaching must succeed");

        assert_eq!(token, None);
        assert_eq!(device.id, existing_id);
        assert_eq!(device.expires_at, target_expiry);
    }

    #[tokio::test]
    async fn attach_replaces_a_device_token_that_resolves_to_nothing() {
        let scope = scope();
        let target = session(&scope, Uuid::new_v4());
        let target_id = target.id;

        let mut devices = devices_resolving(device(&scope));
        devices
            .expect_create()
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));
        devices
            .expect_delete_expired()
            .returning(|_, _| Box::pin(async { Ok(0) }));
        let mut sessions = sessions_finding(target);
        sessions
            .expect_attach_device()
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let service = service(
            devices,
            sessions,
            MockUserRepository::new(),
            MockRealmRepository::new(),
            resumption(),
        );

        let (_, token) = service
            .attach_session_to_device(Some("forged"), &scope, target_id)
            .await
            .expect("attaching must succeed");

        assert_eq!(token.as_deref(), Some("fresh-secret"));
    }

    #[tokio::test]
    async fn attach_of_a_session_of_another_realm_is_refused() {
        let scope = scope();
        let mut target = session(&scope, Uuid::new_v4());
        target.realm_id = Uuid::new_v4();
        let target_id = target.id;

        let mut devices = MockDeviceSessionRepository::new();
        devices.expect_create().never();
        let service = service(
            devices,
            sessions_finding(target),
            MockUserRepository::new(),
            MockRealmRepository::new(),
            resumption(),
        );

        let result = service
            .attach_session_to_device(None, &scope, target_id)
            .await;

        assert!(matches!(result, Err(CoreError::NotFound)));
    }
}
