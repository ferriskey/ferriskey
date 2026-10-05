use std::sync::Arc;

use ferriskey_authz::FerriskeyPolicy;
use ferriskey_domain::auth::Identity;
use ferriskey_domain::client::ports::ClientRepository;
use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::common::pagination::{Page, PageRequest};
use ferriskey_domain::common::policies::ensure_policy;
use ferriskey_domain::realm::ports::RealmRepository;
use ferriskey_domain::realm::scope::RealmScope;
use ferriskey_domain::user::ports::{UserRepository, UserRoleRepository};

use crate::entities::SecurityEvent;
use crate::hashing::{VerifyResult, verify_chain};
use crate::ports::{SecurityEventPolicy, SecurityEventRepository, SecurityEventService};
use crate::value_objects::{SecurityEventFilter, SecurityEventSortField};

#[derive(Clone, Debug)]
pub struct SecurityEventServiceImpl<R, U, C, UR, SE>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    SE: SecurityEventRepository,
{
    pub(crate) realm_repository: Arc<R>,
    pub(crate) security_event_repository: Arc<SE>,
    pub(crate) policy: Arc<FerriskeyPolicy<U, C, UR>>,
}

impl<R, U, C, UR, SE> SecurityEventServiceImpl<R, U, C, UR, SE>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    SE: SecurityEventRepository,
{
    pub fn new(
        realm_repository: Arc<R>,
        security_event_repository: Arc<SE>,
        policy: Arc<FerriskeyPolicy<U, C, UR>>,
    ) -> Self {
        Self {
            realm_repository,
            security_event_repository,
            policy,
        }
    }
}

impl<R, U, C, UR, SE> SecurityEventService for SecurityEventServiceImpl<R, U, C, UR, SE>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    SE: SecurityEventRepository,
{
    async fn list_events(
        &self,
        identity: Identity,
        realm_name: String,
        request: PageRequest<SecurityEventFilter, SecurityEventSortField>,
    ) -> Result<Page<SecurityEvent>, CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &realm_name).await?;

        ensure_policy(
            self.policy.can_view_events(&identity, scope.realm()).await,
            "insufficient permissions",
        )?;

        self.security_event_repository.list(&scope, &request).await
    }

    async fn verify_realm_chain(
        &self,
        identity: Identity,
        realm_name: String,
    ) -> Result<VerifyResult, CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &realm_name).await?;

        ensure_policy(
            self.policy
                .can_export_events(&identity, scope.realm())
                .await,
            "insufficient permissions",
        )?;

        let events = self
            .security_event_repository
            .get_events_ordered_for_verification(scope.id())
            .await?;

        Ok(verify_chain(&events))
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use ferriskey_domain::client::ports::MockClientRepository;
    use ferriskey_domain::common::pagination::{Sort, SortOrder};
    use ferriskey_domain::realm::{Realm, ports::MockRealmRepository};
    use ferriskey_domain::role::entities::Role;
    use ferriskey_domain::user::{
        entities::User,
        ports::{MockUserRepository, MockUserRoleRepository},
    };
    use uuid::Uuid;

    use super::*;
    use crate::entities::{EventStatus, SecurityEventType};
    use crate::ports::MockSecurityEventRepository;

    type TestService = SecurityEventServiceImpl<
        MockRealmRepository,
        MockUserRepository,
        MockClientRepository,
        MockUserRoleRepository,
        MockSecurityEventRepository,
    >;

    fn test_realm() -> Realm {
        Realm {
            id: Uuid::new_v4().into(),
            name: "test-realm".to_string(),
            display_name: None,
            settings: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn test_user(realm: &Realm) -> User {
        User {
            id: Uuid::new_v4(),
            realm_id: realm.id,
            username: "viewer".to_string(),
            firstname: None,
            lastname: None,
            email: None,
            email_verified: true,
            enabled: true,
            roles: None,
            realm: Some(realm.clone()),
            client_id: None,
            required_actions: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            failed_login_attempts: 0,
            locked_until: None,
            locale: None,
        }
    }

    fn role(realm: &Realm, permissions: &[&str]) -> Role {
        Role {
            id: Uuid::new_v4(),
            name: "events".to_string(),
            description: None,
            permissions: permissions.iter().map(ToString::to_string).collect(),
            realm_id: realm.id,
            client_id: None,
            client: None,
            require_mfa: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn build_service(
        realm: &Realm,
        permissions: &'static [&'static str],
        event_repository: MockSecurityEventRepository,
    ) -> TestService {
        let mut realm_repository = MockRealmRepository::new();
        let found = realm.clone();
        realm_repository.expect_get_by_name().returning(move |_| {
            let realm = found.clone();
            Box::pin(async move { Ok(Some(realm)) })
        });

        let mut user_role_repository = MockUserRoleRepository::new();
        let owner = realm.clone();
        user_role_repository
            .expect_get_user_roles()
            .returning(move |_| {
                let roles = vec![role(&owner, permissions)];
                Box::pin(async move { Ok(roles) })
            });

        SecurityEventServiceImpl::new(
            Arc::new(realm_repository),
            Arc::new(event_repository),
            Arc::new(FerriskeyPolicy::new(
                Arc::new(MockUserRepository::new()),
                Arc::new(MockClientRepository::new()),
                Arc::new(user_role_repository),
            )),
        )
    }

    fn list_request() -> PageRequest<SecurityEventFilter, SecurityEventSortField> {
        PageRequest {
            sort: Sort {
                field: SecurityEventSortField::EventType,
                order: SortOrder::Asc,
            },
            filter: SecurityEventFilter {
                status: Some(EventStatus::Failure),
                event_types: Some(vec![SecurityEventType::LoginFailure]),
                ip_address: Some("10.0".to_string()),
                ..SecurityEventFilter::default()
            },
            ..PageRequest::default()
        }
    }

    #[tokio::test]
    async fn list_events_refuses_a_caller_without_view_rights_before_listing() {
        let realm = test_realm();
        let user = test_user(&realm);

        let mut event_repository = MockSecurityEventRepository::new();
        event_repository.expect_list().never();

        let service = build_service(&realm, &["view_realm"], event_repository);

        let result = service
            .list_events(Identity::User(user), realm.name.clone(), list_request())
            .await;

        assert!(matches!(result, Err(CoreError::Forbidden(_))), "{result:?}");
    }

    #[tokio::test]
    async fn list_events_passes_the_realm_scope_and_the_request_through() {
        let realm = test_realm();
        let user = test_user(&realm);
        let realm_id = realm.id;
        let listed = SecurityEvent::without_actor(
            realm_id,
            SecurityEventType::LoginFailure,
            EventStatus::Failure,
        );
        let expected = listed.clone();

        let mut event_repository = MockSecurityEventRepository::new();
        event_repository
            .expect_list()
            .withf(move |scope, request| scope.id() == realm_id && *request == list_request())
            .times(1)
            .return_once(move |_, request| {
                let page = Page::new(vec![listed], 1, request.page, request.limit);
                Box::pin(async move { Ok(page) })
            });

        let service = build_service(&realm, &["view_events"], event_repository);

        let page = service
            .list_events(Identity::User(user), realm.name.clone(), list_request())
            .await
            .expect("listed");

        assert_eq!(page.data(), [expected]);
        assert_eq!(page.metadata().total, 1);
    }
}
