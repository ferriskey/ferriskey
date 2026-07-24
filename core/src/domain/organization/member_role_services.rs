use std::sync::Arc;

use ferriskey_domain::realm::RealmId;

use crate::domain::{
    authentication::value_objects::Identity,
    client::ports::ClientRepository,
    common::{
        entities::app_errors::CoreError,
        policies::{FerriskeyPolicy, ensure_policy},
    },
    organization::ports::{
        AssignMemberRoleInput, ListMemberRolesInput, Organization, OrganizationId,
        OrganizationMemberRepository, OrganizationMemberRoleRepository,
        OrganizationMemberRoleService, OrganizationPolicy, OrganizationRepository,
        RevokeMemberRoleInput,
    },
    realm::ports::RealmRepository,
    role::entities::Role,
    user::ports::{UserRepository, UserRoleRepository},
};

/// Business logic for roles scoped to an organization membership. Kept as a dedicated service so
/// the existing organization/group services (and their generics) stay untouched.
#[derive(Clone, Debug)]
pub struct OrganizationMemberRoleServiceImpl<R, U, C, UR, OR, OMR, OMRR>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    OR: OrganizationRepository,
    OMR: OrganizationMemberRepository,
    OMRR: OrganizationMemberRoleRepository,
{
    pub(crate) realm_repository: Arc<R>,
    pub(crate) user_role_repository: Arc<UR>,
    pub(crate) organization_repository: Arc<OR>,
    pub(crate) organization_member_repository: Arc<OMR>,
    pub(crate) organization_member_role_repository: Arc<OMRR>,
    pub(crate) policy: Arc<FerriskeyPolicy<U, C, UR>>,
}

impl<R, U, C, UR, OR, OMR, OMRR> OrganizationMemberRoleServiceImpl<R, U, C, UR, OR, OMR, OMRR>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    OR: OrganizationRepository,
    OMR: OrganizationMemberRepository,
    OMRR: OrganizationMemberRoleRepository,
{
    pub fn new(
        realm_repository: Arc<R>,
        user_role_repository: Arc<UR>,
        organization_repository: Arc<OR>,
        organization_member_repository: Arc<OMR>,
        organization_member_role_repository: Arc<OMRR>,
        policy: Arc<FerriskeyPolicy<U, C, UR>>,
    ) -> Self {
        Self {
            realm_repository,
            user_role_repository,
            organization_repository,
            organization_member_repository,
            organization_member_role_repository,
            policy,
        }
    }

    async fn get_org_for_realm_name(
        &self,
        realm_name: String,
        organization_id: OrganizationId,
    ) -> Result<Organization, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        self.get_org_for_realm(organization_id, realm.id).await
    }

    async fn get_org_for_realm(
        &self,
        organization_id: OrganizationId,
        realm_id: RealmId,
    ) -> Result<Organization, CoreError> {
        let org = self
            .organization_repository
            .get_organization_by_id(organization_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        if org.realm_id != realm_id {
            return Err(CoreError::NotFound);
        }

        Ok(org)
    }

    /// Resolve the `organization_members` row id for `user_id` within `org`, or `NotFound`.
    async fn resolve_member_id(
        &self,
        org: &Organization,
        user_id: uuid::Uuid,
    ) -> Result<uuid::Uuid, CoreError> {
        let member = self
            .organization_member_repository
            .get_member(org.id, user_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        Ok(member.id)
    }
}

impl<R, U, C, UR, OR, OMR, OMRR> OrganizationMemberRoleService
    for OrganizationMemberRoleServiceImpl<R, U, C, UR, OR, OMR, OMRR>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    OR: OrganizationRepository,
    OMR: OrganizationMemberRepository,
    OMRR: OrganizationMemberRoleRepository,
{
    async fn assign_role(
        &self,
        identity: Identity,
        input: AssignMemberRoleInput,
    ) -> Result<(), CoreError> {
        let org = self
            .get_org_for_realm_name(input.realm_name, input.organization_id)
            .await?;

        ensure_policy(
            self.policy.can_manage_members(&identity, &org).await,
            "insufficient permissions to manage member roles",
        )?;

        let member_id = self.resolve_member_id(&org, input.user_id).await?;
        self.organization_member_role_repository
            .assign_role(member_id, input.role_id)
            .await
    }

    async fn revoke_role(
        &self,
        identity: Identity,
        input: RevokeMemberRoleInput,
    ) -> Result<(), CoreError> {
        let org = self
            .get_org_for_realm_name(input.realm_name, input.organization_id)
            .await?;

        ensure_policy(
            self.policy.can_manage_members(&identity, &org).await,
            "insufficient permissions to manage member roles",
        )?;

        let member_id = self.resolve_member_id(&org, input.user_id).await?;
        self.organization_member_role_repository
            .revoke_role(member_id, input.role_id)
            .await
    }

    async fn list_roles(
        &self,
        identity: Identity,
        input: ListMemberRolesInput,
    ) -> Result<Vec<Role>, CoreError> {
        let org = self
            .get_org_for_realm_name(input.realm_name, input.organization_id)
            .await?;

        ensure_policy(
            self.policy.can_view_organization(&identity, &org).await,
            "insufficient permissions to view member roles",
        )?;

        let member_id = self.resolve_member_id(&org, input.user_id).await?;
        let role_ids = self
            .organization_member_role_repository
            .list_role_ids(member_id)
            .await?;

        self.user_role_repository.get_roles_by_ids(role_ids).await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::Utc;
    use uuid::Uuid;

    use ferriskey_domain::realm::RealmId;
    use ferriskey_organization::{
        MockOrganizationMemberRepository, MockOrganizationMemberRoleRepository,
        MockOrganizationRepository, Organization, OrganizationId, OrganizationMember,
    };

    use crate::domain::{
        authentication::value_objects::Identity,
        client::ports::MockClientRepository,
        common::{entities::app_errors::CoreError, policies::FerriskeyPolicy},
        realm::{entities::Realm, ports::MockRealmRepository},
        role::entities::Role,
        user::{
            entities::User,
            ports::{MockUserRepository, MockUserRoleRepository},
        },
    };

    use super::*;

    fn make_realm(id: RealmId) -> Realm {
        Realm {
            id,
            name: "test-realm".to_string(),
            display_name: None,
            settings: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn make_user(realm: &Realm) -> User {
        User {
            id: Uuid::new_v4(),
            realm_id: realm.id,
            client_id: None,
            username: "admin".to_string(),
            firstname: Some("Admin".to_string()),
            lastname: Some("User".to_string()),
            email: Some("admin@test.com".to_string()),
            email_verified: true,
            enabled: true,
            roles: None,
            realm: Some(realm.clone()),
            required_actions: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            failed_login_attempts: 0,
            locked_until: None,
        }
    }

    fn make_role(realm_id: RealmId, permission: &str) -> Role {
        Role {
            id: Uuid::new_v4(),
            name: "admin".to_string(),
            description: None,
            permissions: vec![permission.to_string()],
            realm_id,
            client_id: None,
            client: None,
            require_mfa: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn make_org(realm_id: RealmId) -> Organization {
        Organization {
            id: OrganizationId::new(Uuid::new_v4()),
            realm_id,
            name: "Test Org".to_string(),
            alias: "test-org".to_string(),
            domain: None,
            redirect_url: None,
            description: None,
            enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    type TestService = OrganizationMemberRoleServiceImpl<
        MockRealmRepository,
        MockUserRepository,
        MockClientRepository,
        MockUserRoleRepository,
        MockOrganizationRepository,
        MockOrganizationMemberRepository,
        MockOrganizationMemberRoleRepository,
    >;

    fn build_service(
        realm_repo: MockRealmRepository,
        user_repo: MockUserRepository,
        user_role_repo: MockUserRoleRepository,
        org_repo: MockOrganizationRepository,
        member_repo: MockOrganizationMemberRepository,
        member_role_repo: MockOrganizationMemberRoleRepository,
    ) -> TestService {
        let user_arc = Arc::new(user_repo);
        let user_role_arc = Arc::new(user_role_repo);
        let policy = Arc::new(FerriskeyPolicy::new(
            user_arc.clone(),
            Arc::new(MockClientRepository::new()),
            user_role_arc.clone(),
        ));

        OrganizationMemberRoleServiceImpl::new(
            Arc::new(realm_repo),
            user_role_arc,
            Arc::new(org_repo),
            Arc::new(member_repo),
            Arc::new(member_role_repo),
            policy,
        )
    }

    fn realm_repo_returning(realm_id: RealmId) -> MockRealmRepository {
        let mut realm_repo = MockRealmRepository::new();
        realm_repo.expect_get_by_name().returning(move |_| {
            let r = make_realm(realm_id);
            Box::pin(async move { Ok(Some(r)) })
        });
        realm_repo
    }

    fn admin_user_repo(admin: User) -> MockUserRepository {
        let mut user_repo = MockUserRepository::new();
        user_repo.expect_get_by_id().returning(move |_| {
            let u = admin.clone();
            Box::pin(async move { Ok(u) })
        });
        user_repo
    }

    fn admin_role_repo(realm_id: RealmId, permission: &'static str) -> MockUserRoleRepository {
        let mut user_role_repo = MockUserRoleRepository::new();
        user_role_repo.expect_get_user_roles().returning(move |_| {
            let role = make_role(realm_id, permission);
            Box::pin(async move { Ok(vec![role]) })
        });
        user_role_repo
    }

    #[tokio::test]
    async fn assign_role_resolves_member_and_persists() {
        let realm_id = RealmId::new(Uuid::new_v4());
        let realm = make_realm(realm_id);
        let admin = make_user(&realm);
        let identity = Identity::User(admin.clone());
        let org = make_org(realm_id);
        let org_id = org.id;
        let target_user_id = Uuid::new_v4();
        let member = OrganizationMember {
            id: Uuid::new_v4(),
            organization_id: org_id,
            user_id: target_user_id,
            created_at: Utc::now(),
        };
        let member_row_id = member.id;
        let role_id = Uuid::new_v4();

        let mut org_repo = MockOrganizationRepository::new();
        org_repo
            .expect_get_organization_by_id()
            .return_once(move |_| Box::pin(async move { Ok(Some(org)) }));

        let mut member_repo = MockOrganizationMemberRepository::new();
        member_repo
            .expect_get_member()
            .return_once(move |_, _| Box::pin(async move { Ok(Some(member)) }));

        let mut member_role_repo = MockOrganizationMemberRoleRepository::new();
        member_role_repo
            .expect_assign_role()
            .withf(move |mid, rid| *mid == member_row_id && *rid == role_id)
            .return_once(|_, _| Box::pin(async { Ok(()) }));

        let service = build_service(
            realm_repo_returning(realm_id),
            admin_user_repo(admin),
            admin_role_repo(realm_id, "manage_users"),
            org_repo,
            member_repo,
            member_role_repo,
        );

        let result = service
            .assign_role(
                identity,
                AssignMemberRoleInput {
                    realm_name: "test-realm".to_string(),
                    organization_id: org_id,
                    user_id: target_user_id,
                    role_id,
                },
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn assign_role_returns_not_found_when_membership_missing() {
        let realm_id = RealmId::new(Uuid::new_v4());
        let realm = make_realm(realm_id);
        let admin = make_user(&realm);
        let identity = Identity::User(admin.clone());
        let org = make_org(realm_id);
        let org_id = org.id;

        let mut org_repo = MockOrganizationRepository::new();
        org_repo
            .expect_get_organization_by_id()
            .return_once(move |_| Box::pin(async move { Ok(Some(org)) }));

        let mut member_repo = MockOrganizationMemberRepository::new();
        member_repo
            .expect_get_member()
            .return_once(|_, _| Box::pin(async { Ok(None) }));

        let service = build_service(
            realm_repo_returning(realm_id),
            admin_user_repo(admin),
            admin_role_repo(realm_id, "manage_users"),
            org_repo,
            member_repo,
            MockOrganizationMemberRoleRepository::new(),
        );

        let result = service
            .assign_role(
                identity,
                AssignMemberRoleInput {
                    realm_name: "test-realm".to_string(),
                    organization_id: org_id,
                    user_id: Uuid::new_v4(),
                    role_id: Uuid::new_v4(),
                },
            )
            .await;

        assert!(matches!(result, Err(CoreError::NotFound)));
    }

    #[tokio::test]
    async fn assign_role_forbidden_without_permission() {
        let realm_id = RealmId::new(Uuid::new_v4());
        let realm = make_realm(realm_id);
        let admin = make_user(&realm);
        let identity = Identity::User(admin.clone());
        let org = make_org(realm_id);
        let org_id = org.id;

        let mut org_repo = MockOrganizationRepository::new();
        org_repo
            .expect_get_organization_by_id()
            .return_once(move |_| Box::pin(async move { Ok(Some(org)) }));

        let service = build_service(
            realm_repo_returning(realm_id),
            admin_user_repo(admin),
            admin_role_repo(realm_id, "view_users"), // lacks manage rights
            org_repo,
            MockOrganizationMemberRepository::new(),
            MockOrganizationMemberRoleRepository::new(),
        );

        let result = service
            .assign_role(
                identity,
                AssignMemberRoleInput {
                    realm_name: "test-realm".to_string(),
                    organization_id: org_id,
                    user_id: Uuid::new_v4(),
                    role_id: Uuid::new_v4(),
                },
            )
            .await;

        assert!(matches!(result, Err(CoreError::Forbidden(_))));
    }
}
