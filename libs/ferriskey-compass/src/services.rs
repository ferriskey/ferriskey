use std::sync::Arc;

use ferriskey_authz::FerriskeyPolicy;
use ferriskey_domain::auth::Identity;
use ferriskey_domain::client::ports::ClientRepository;
use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::common::pagination::{Page, PageRequest};
use ferriskey_domain::common::policies::ensure_policy;
use ferriskey_domain::realm::ports::RealmRepository;
use ferriskey_domain::realm::scope::{RealmScope, UnscopedOption};
use ferriskey_domain::user::ports::{UserRepository, UserRoleRepository};
use uuid::Uuid;

use crate::{
    entities::CompassFlow,
    ports::{CompassFlowRepository, CompassFlowStepRepository, CompassPolicy, CompassService},
    value_objects::{
        DailyActivityStats, DailyActivityStatsFilter, FlowFilter, FlowSortField, FlowStats,
    },
};

#[derive(Clone, Debug)]
pub struct CompassServiceImpl<R, U, C, UR, FR, FS>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    FR: CompassFlowRepository,
    FS: CompassFlowStepRepository,
{
    pub(crate) realm_repository: Arc<R>,
    pub(crate) flow_repository: Arc<FR>,
    pub(crate) step_repository: Arc<FS>,
    pub(crate) policy: Arc<FerriskeyPolicy<U, C, UR>>,
}

impl<R, U, C, UR, FR, FS> CompassServiceImpl<R, U, C, UR, FR, FS>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    FR: CompassFlowRepository,
    FS: CompassFlowStepRepository,
{
    pub fn new(
        realm_repository: Arc<R>,
        flow_repository: Arc<FR>,
        step_repository: Arc<FS>,
        policy: Arc<FerriskeyPolicy<U, C, UR>>,
    ) -> Self {
        Self {
            realm_repository,
            flow_repository,
            step_repository,
            policy,
        }
    }
}

impl<R, U, C, UR, FR, FS> CompassService for CompassServiceImpl<R, U, C, UR, FR, FS>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    FR: CompassFlowRepository,
    FS: CompassFlowStepRepository,
{
    async fn list_flows(
        &self,
        identity: Identity,
        realm_name: String,
        request: PageRequest<FlowFilter, FlowSortField>,
    ) -> Result<Page<CompassFlow>, CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &realm_name).await?;

        ensure_policy(
            self.policy.can_view_flows(&identity, scope.realm()).await,
            "insufficient permissions",
        )?;

        self.flow_repository.list(&scope, &request).await
    }

    async fn get_flow(
        &self,
        identity: Identity,
        realm_name: String,
        flow_id: Uuid,
    ) -> Result<CompassFlow, CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &realm_name).await?;

        ensure_policy(
            self.policy.can_view_flows(&identity, scope.realm()).await,
            "insufficient permissions",
        )?;

        let mut flow = self
            .flow_repository
            .get_flow_by_id(flow_id)
            .await?
            .in_realm(&scope)?
            .ok_or(CoreError::NotFound)?
            .into_inner();

        let steps = self.step_repository.get_steps_for_flow(flow_id).await?;
        flow.steps = steps;

        Ok(flow)
    }

    async fn get_stats(
        &self,
        identity: Identity,
        realm_name: String,
    ) -> Result<FlowStats, CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &realm_name).await?;

        ensure_policy(
            self.policy.can_view_flows(&identity, scope.realm()).await,
            "insufficient permissions",
        )?;

        let stats = self.flow_repository.get_stats(scope.id()).await?;

        Ok(stats)
    }

    async fn get_daily_activity_stats(
        &self,
        identity: Identity,
        realm_name: String,
        filter: DailyActivityStatsFilter,
    ) -> Result<Vec<DailyActivityStats>, CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &realm_name).await?;

        ensure_policy(
            self.policy.can_view_flows(&identity, scope.realm()).await,
            "insufficient permissions",
        )?;

        let stats = self
            .flow_repository
            .get_daily_activity_stats(scope.id(), filter)
            .await?;

        Ok(stats)
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

    use super::*;
    use crate::entities::FlowStatus;
    use crate::ports::{MockCompassFlowRepository, MockCompassFlowStepRepository};

    type TestService = CompassServiceImpl<
        MockRealmRepository,
        MockUserRepository,
        MockClientRepository,
        MockUserRoleRepository,
        MockCompassFlowRepository,
        MockCompassFlowStepRepository,
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
            name: "flows".to_string(),
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
        flow_repository: MockCompassFlowRepository,
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

        CompassServiceImpl::new(
            Arc::new(realm_repository),
            Arc::new(flow_repository),
            Arc::new(MockCompassFlowStepRepository::new()),
            Arc::new(FerriskeyPolicy::new(
                Arc::new(MockUserRepository::new()),
                Arc::new(MockClientRepository::new()),
                Arc::new(user_role_repository),
            )),
        )
    }

    fn list_request() -> PageRequest<FlowFilter, FlowSortField> {
        PageRequest {
            sort: Sort {
                field: FlowSortField::DurationMs,
                order: SortOrder::Asc,
            },
            filter: FlowFilter {
                status: Some(FlowStatus::Failure),
                ip_address: Some("10.0".to_string()),
                ..FlowFilter::default()
            },
            ..PageRequest::default()
        }
    }

    #[tokio::test]
    async fn list_flows_refuses_a_caller_without_view_rights_before_listing() {
        let realm = test_realm();
        let user = test_user(&realm);

        let mut flow_repository = MockCompassFlowRepository::new();
        flow_repository.expect_list().never();

        let service = build_service(&realm, &["view_realm"], flow_repository);

        let result = service
            .list_flows(Identity::User(user), realm.name.clone(), list_request())
            .await;

        assert!(matches!(result, Err(CoreError::Forbidden(_))), "{result:?}");
    }

    #[tokio::test]
    async fn list_flows_passes_the_realm_scope_and_the_request_through() {
        let realm = test_realm();
        let user = test_user(&realm);
        let realm_id = realm.id;
        let listed = CompassFlow::new(realm_id, None, "password".to_string(), None, None);
        let expected = listed.clone();

        let mut flow_repository = MockCompassFlowRepository::new();
        flow_repository
            .expect_list()
            .withf(move |scope, request| scope.id() == realm_id && *request == list_request())
            .times(1)
            .return_once(move |_, request| {
                let page = Page::new(vec![listed], 1, request.page, request.limit);
                Box::pin(async move { Ok(page) })
            });

        let service = build_service(&realm, &["view_events"], flow_repository);

        let page = service
            .list_flows(Identity::User(user), realm.name.clone(), list_request())
            .await
            .expect("listed");

        assert_eq!(page.data(), [expected]);
        assert_eq!(page.metadata().total, 1);
    }
}
