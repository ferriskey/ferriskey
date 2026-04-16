use std::sync::Arc;

use ferriskey_organization::{
    OrganizationAttribute, OrganizationConfig, OrganizationValidationError,
    validate_membership_realms,
};

use crate::domain::{
    authentication::value_objects::Identity,
    client::ports::ClientRepository,
    common::{
        entities::app_errors::CoreError,
        policies::{FerriskeyPolicy, ensure_policy},
    },
    organization::ports::{
        AddOrganizationMemberInput, CreateOrganizationInput, DeleteOrganizationAttributeInput,
        DeleteOrganizationInput, GetOrganizationInput, ListOrganizationAttributesInput,
        ListOrganizationMembersInput, ListOrganizationsInput, ListUserOrganizationsInput,
        Organization, OrganizationAttributeRepository, OrganizationId, OrganizationMember,
        OrganizationMemberRepository, OrganizationPolicy, OrganizationRepository,
        OrganizationService, RemoveOrganizationMemberInput, UpdateOrganizationInput,
        UpdateOrganizationParams, UpsertOrganizationAttributeInput,
    },
    realm::ports::RealmRepository,
    user::ports::{UserRepository, UserRoleRepository},
};

#[derive(Clone, Debug)]
pub struct OrganizationServiceImpl<R, U, C, UR, OR, OAR, OMR>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    OR: OrganizationRepository,
    OAR: OrganizationAttributeRepository,
    OMR: OrganizationMemberRepository,
{
    pub(crate) realm_repository: Arc<R>,
    pub(crate) user_repository: Arc<U>,
    pub(crate) organization_repository: Arc<OR>,
    pub(crate) organization_attribute_repository: Arc<OAR>,
    pub(crate) organization_member_repository: Arc<OMR>,
    pub(crate) policy: Arc<FerriskeyPolicy<U, C, UR>>,
}

impl<R, U, C, UR, OR, OAR, OMR> OrganizationServiceImpl<R, U, C, UR, OR, OAR, OMR>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    OR: OrganizationRepository,
    OAR: OrganizationAttributeRepository,
    OMR: OrganizationMemberRepository,
{
    pub fn new(
        realm_repository: Arc<R>,
        user_repository: Arc<U>,
        organization_repository: Arc<OR>,
        organization_attribute_repository: Arc<OAR>,
        organization_member_repository: Arc<OMR>,
        policy: Arc<FerriskeyPolicy<U, C, UR>>,
    ) -> Self {
        Self {
            realm_repository,
            user_repository,
            organization_repository,
            organization_attribute_repository,
            organization_member_repository,
            policy,
        }
    }

    async fn get_realm_by_name(
        &self,
        realm_name: String,
    ) -> Result<crate::domain::realm::entities::Realm, CoreError> {
        self.realm_repository
            .get_by_name(&realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)
    }

    async fn get_org_for_realm(
        &self,
        organization_id: OrganizationId,
        realm_id: ferriskey_domain::realm::RealmId,
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
}

impl<R, U, C, UR, OR, OAR, OMR> OrganizationService
    for OrganizationServiceImpl<R, U, C, UR, OR, OAR, OMR>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    OR: OrganizationRepository,
    OAR: OrganizationAttributeRepository,
    OMR: OrganizationMemberRepository,
{
    async fn create_organization(
        &self,
        identity: Identity,
        input: CreateOrganizationInput,
    ) -> Result<Organization, CoreError> {
        let realm = self.get_realm_by_name(input.realm_name).await?;

        ensure_policy(
            self.policy
                .can_create_organization(&identity, realm.id)
                .await,
            "insufficient permissions to create organization",
        )?;

        if self
            .organization_repository
            .exists_organization_by_realm_and_alias(realm.id, &input.alias)
            .await?
        {
            return Err(CoreError::AlreadyExists);
        }

        let org_config = OrganizationConfig {
            realm_id: realm.id,
            name: input.name,
            alias: input.alias,
            domain: input.domain,
            redirect_url: input.redirect_url,
            description: input.description,
            enabled: input.enabled,
        };

        let org = ferriskey_organization::Organization::new(org_config)
            .map_err(|_| CoreError::Invalid)?;

        self.organization_repository
            .create_organization(ferriskey_organization::CreateOrganizationParams {
                realm_id: org.realm_id,
                name: org.name.clone(),
                alias: org.alias.clone(),
                domain: org.domain.clone(),
                redirect_url: org.redirect_url.clone(),
                description: org.description.clone(),
                enabled: org.enabled,
            })
            .await
    }

    async fn get_organization(
        &self,
        identity: Identity,
        input: GetOrganizationInput,
    ) -> Result<Organization, CoreError> {
        let realm = self.get_realm_by_name(input.realm_name).await?;
        let org = self
            .get_org_for_realm(input.organization_id, realm.id)
            .await?;

        ensure_policy(
            self.policy.can_view_organization(&identity, &org).await,
            "insufficient permissions to view organization",
        )?;

        Ok(org)
    }

    async fn list_organizations(
        &self,
        identity: Identity,
        input: ListOrganizationsInput,
    ) -> Result<Vec<Organization>, CoreError> {
        let realm = self.get_realm_by_name(input.realm_name).await?;

        ensure_policy(
            self.policy
                .can_create_organization(&identity, realm.id)
                .await,
            "insufficient permissions to list organizations",
        )?;

        self.organization_repository
            .list_organizations_by_realm(realm.id)
            .await
    }

    async fn update_organization(
        &self,
        identity: Identity,
        input: UpdateOrganizationInput,
    ) -> Result<Organization, CoreError> {
        let realm = self.get_realm_by_name(input.realm_name).await?;
        let org = self
            .get_org_for_realm(input.organization_id, realm.id)
            .await?;

        ensure_policy(
            self.policy.can_update_organization(&identity, &org).await,
            "insufficient permissions to update organization",
        )?;

        if let Some(ref new_alias) = input.alias
            && *new_alias != org.alias
            && self
                .organization_repository
                .exists_organization_by_realm_and_alias(realm.id, new_alias)
                .await?
        {
            return Err(CoreError::AlreadyExists);
        }

        let params = UpdateOrganizationParams {
            name: input.name,
            alias: input.alias,
            domain: input.domain,
            redirect_url: input.redirect_url,
            description: input.description,
            enabled: input.enabled,
        };

        self.organization_repository
            .update_organization(org.id, params)
            .await
    }

    async fn delete_organization(
        &self,
        identity: Identity,
        input: DeleteOrganizationInput,
    ) -> Result<(), CoreError> {
        let realm = self.get_realm_by_name(input.realm_name).await?;
        let org = self
            .get_org_for_realm(input.organization_id, realm.id)
            .await?;

        ensure_policy(
            self.policy.can_delete_organization(&identity, &org).await,
            "insufficient permissions to delete organization",
        )?;

        self.organization_repository
            .delete_organization(org.id)
            .await
    }

    async fn list_attributes(
        &self,
        identity: Identity,
        input: ListOrganizationAttributesInput,
    ) -> Result<Vec<OrganizationAttribute>, CoreError> {
        let realm = self.get_realm_by_name(input.realm_name).await?;
        let org = self
            .get_org_for_realm(input.organization_id, realm.id)
            .await?;

        ensure_policy(
            self.policy.can_view_organization(&identity, &org).await,
            "insufficient permissions to list organization attributes",
        )?;

        self.organization_attribute_repository
            .list_attributes(org.id)
            .await
    }

    async fn upsert_attribute(
        &self,
        identity: Identity,
        input: UpsertOrganizationAttributeInput,
    ) -> Result<OrganizationAttribute, CoreError> {
        let realm = self.get_realm_by_name(input.realm_name).await?;
        let org = self
            .get_org_for_realm(input.organization_id, realm.id)
            .await?;

        ensure_policy(
            self.policy.can_update_organization(&identity, &org).await,
            "insufficient permissions to upsert organization attribute",
        )?;

        // Validate attribute key and value via domain constructor
        OrganizationAttribute::new(org.id, input.key.clone(), input.value.clone()).map_err(
            |e| match e {
                OrganizationValidationError::EmptyAttributeKey
                | OrganizationValidationError::AttributeKeyTooLong
                | OrganizationValidationError::EmptyAttributeValue => CoreError::Invalid,
                _ => CoreError::Invalid,
            },
        )?;

        self.organization_attribute_repository
            .upsert_attribute(org.id, input.key, input.value)
            .await
    }

    async fn delete_attribute(
        &self,
        identity: Identity,
        input: DeleteOrganizationAttributeInput,
    ) -> Result<(), CoreError> {
        let realm = self.get_realm_by_name(input.realm_name).await?;
        let org = self
            .get_org_for_realm(input.organization_id, realm.id)
            .await?;

        ensure_policy(
            self.policy.can_update_organization(&identity, &org).await,
            "insufficient permissions to delete organization attribute",
        )?;

        self.organization_attribute_repository
            .delete_attribute(org.id, &input.key)
            .await
    }

    async fn add_member(
        &self,
        identity: Identity,
        input: AddOrganizationMemberInput,
    ) -> Result<OrganizationMember, CoreError> {
        let realm = self.get_realm_by_name(input.realm_name).await?;
        let org = self
            .get_org_for_realm(input.organization_id, realm.id)
            .await?;

        ensure_policy(
            self.policy.can_manage_members(&identity, &org).await,
            "insufficient permissions to add organization member",
        )?;

        // Disabled organizations cannot accept new members
        if !org.enabled {
            return Err(CoreError::Invalid);
        }

        // Verify user exists and belongs to the same realm as the organization
        let user = self
            .user_repository
            .get_by_id(input.user_id)
            .await
            .map_err(|_| CoreError::UserNotFound)?;

        validate_membership_realms(org.realm_id, user.realm_id).map_err(|_| CoreError::Invalid)?;

        // Reject duplicate memberships
        if self
            .organization_member_repository
            .get_member(org.id, input.user_id)
            .await?
            .is_some()
        {
            return Err(CoreError::AlreadyExists);
        }

        self.organization_member_repository
            .add_member(org.id, input.user_id)
            .await
    }

    async fn remove_member(
        &self,
        identity: Identity,
        input: RemoveOrganizationMemberInput,
    ) -> Result<(), CoreError> {
        let realm = self.get_realm_by_name(input.realm_name).await?;
        let org = self
            .get_org_for_realm(input.organization_id, realm.id)
            .await?;

        ensure_policy(
            self.policy.can_manage_members(&identity, &org).await,
            "insufficient permissions to remove organization member",
        )?;

        // Verify the membership exists before attempting removal
        self.organization_member_repository
            .get_member(org.id, input.user_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        self.organization_member_repository
            .remove_member(org.id, input.user_id)
            .await
    }

    async fn list_members(
        &self,
        identity: Identity,
        input: ListOrganizationMembersInput,
    ) -> Result<Vec<OrganizationMember>, CoreError> {
        let realm = self.get_realm_by_name(input.realm_name).await?;
        let org = self
            .get_org_for_realm(input.organization_id, realm.id)
            .await?;

        ensure_policy(
            self.policy.can_view_organization(&identity, &org).await,
            "insufficient permissions to list organization members",
        )?;

        self.organization_member_repository
            .list_members(org.id)
            .await
    }

    async fn list_user_organizations(
        &self,
        identity: Identity,
        input: ListUserOrganizationsInput,
    ) -> Result<Vec<OrganizationMember>, CoreError> {
        let realm = self.get_realm_by_name(input.realm_name).await?;

        ensure_policy(
            self.policy
                .can_create_organization(&identity, realm.id)
                .await,
            "insufficient permissions to list user organizations",
        )?;

        self.organization_member_repository
            .list_organizations_for_user(input.user_id)
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::Utc;
    use uuid::Uuid;

    use ferriskey_domain::realm::RealmId;
    use ferriskey_organization::{
        MockOrganizationAttributeRepository, MockOrganizationMemberRepository,
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

    fn make_realm(id: RealmId, name: &str) -> Realm {
        Realm {
            id,
            name: name.to_string(),
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
            firstname: "Admin".to_string(),
            lastname: "User".to_string(),
            email: "admin@test.com".to_string(),
            email_verified: true,
            enabled: true,
            roles: None,
            realm: Some(realm.clone()),
            required_actions: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn make_role_with_permission(realm_id: RealmId, permission: &str) -> Role {
        Role {
            id: Uuid::new_v4(),
            name: "admin".to_string(),
            description: None,
            permissions: vec![permission.to_string()],
            realm_id,
            client_id: None,
            client: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn make_organization(realm_id: RealmId) -> Organization {
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

    fn make_member(org_id: OrganizationId, user_id: Uuid) -> OrganizationMember {
        OrganizationMember {
            id: Uuid::new_v4(),
            organization_id: org_id,
            user_id,
            created_at: Utc::now(),
        }
    }

    type TestService = OrganizationServiceImpl<
        MockRealmRepository,
        MockUserRepository,
        MockClientRepository,
        MockUserRoleRepository,
        MockOrganizationRepository,
        MockOrganizationAttributeRepository,
        MockOrganizationMemberRepository,
    >;

    fn build_service(
        realm_repo: MockRealmRepository,
        user_repo: MockUserRepository,
        user_role_repo: MockUserRoleRepository,
        org_repo: MockOrganizationRepository,
        attr_repo: MockOrganizationAttributeRepository,
        member_repo: MockOrganizationMemberRepository,
    ) -> TestService {
        let user_arc = Arc::new(user_repo);
        let policy = Arc::new(FerriskeyPolicy::new(
            user_arc.clone(),
            Arc::new(MockClientRepository::new()),
            Arc::new(user_role_repo),
        ));

        OrganizationServiceImpl::new(
            Arc::new(realm_repo),
            user_arc,
            Arc::new(org_repo),
            Arc::new(attr_repo),
            Arc::new(member_repo),
            policy,
        )
    }

    // ─── previous tests ──────────────────────────────────────────────────────────

    #[tokio::test]
    async fn get_organization_returns_not_found_for_missing_org() {
        let realm_id = RealmId::new(Uuid::new_v4());
        let realm = make_realm(realm_id, "test-realm");
        let user = make_user(&realm);
        let identity = Identity::User(user.clone());
        let org_id = OrganizationId::new(Uuid::new_v4());

        let mut realm_repo = MockRealmRepository::new();
        realm_repo.expect_get_by_name().returning(move |_| {
            let r = make_realm(realm_id, "test-realm");
            Box::pin(async move { Ok(Some(r)) })
        });

        let mut org_repo = MockOrganizationRepository::new();
        org_repo
            .expect_get_organization_by_id()
            .return_once(|_| Box::pin(async { Ok(None) }));

        let mut user_repo = MockUserRepository::new();
        user_repo.expect_get_by_id().returning(move |_| {
            let u = user.clone();
            Box::pin(async move { Ok(u) })
        });

        let mut user_role_repo = MockUserRoleRepository::new();
        user_role_repo
            .expect_get_user_roles()
            .returning(move |_| Box::pin(async { Ok(vec![]) }));

        let service = build_service(
            realm_repo,
            user_repo,
            user_role_repo,
            org_repo,
            MockOrganizationAttributeRepository::new(),
            MockOrganizationMemberRepository::new(),
        );

        let result = service
            .get_organization(
                identity,
                GetOrganizationInput {
                    realm_name: "test-realm".to_string(),
                    organization_id: org_id,
                },
            )
            .await;

        assert!(matches!(result, Err(CoreError::NotFound)));
    }

    #[tokio::test]
    async fn get_organization_returns_forbidden_without_permission() {
        let realm_id = RealmId::new(Uuid::new_v4());
        let realm = make_realm(realm_id, "test-realm");
        let user = make_user(&realm);
        let identity = Identity::User(user.clone());
        let org = make_organization(realm_id);
        let org_id = org.id;

        let mut realm_repo = MockRealmRepository::new();
        realm_repo.expect_get_by_name().returning(move |_| {
            let r = make_realm(realm_id, "test-realm");
            Box::pin(async move { Ok(Some(r)) })
        });

        let mut org_repo = MockOrganizationRepository::new();
        org_repo
            .expect_get_organization_by_id()
            .return_once(move |_| Box::pin(async move { Ok(Some(org)) }));

        let mut user_repo = MockUserRepository::new();
        user_repo.expect_get_by_id().returning(move |_| {
            let u = user.clone();
            Box::pin(async move { Ok(u) })
        });

        let mut user_role_repo = MockUserRoleRepository::new();
        user_role_repo
            .expect_get_user_roles()
            .returning(|_| Box::pin(async { Ok(vec![]) }));

        let service = build_service(
            realm_repo,
            user_repo,
            user_role_repo,
            org_repo,
            MockOrganizationAttributeRepository::new(),
            MockOrganizationMemberRepository::new(),
        );

        let result = service
            .get_organization(
                identity,
                GetOrganizationInput {
                    realm_name: "test-realm".to_string(),
                    organization_id: org_id,
                },
            )
            .await;

        assert!(matches!(result, Err(CoreError::Forbidden(_))));
    }

    #[tokio::test]
    async fn list_attributes_returns_empty_list_for_org_with_no_attributes() {
        let realm_id = RealmId::new(Uuid::new_v4());
        let realm = make_realm(realm_id, "test-realm");
        let user = make_user(&realm);
        let identity = Identity::User(user.clone());
        let org = make_organization(realm_id);
        let org_id = org.id;

        let mut realm_repo = MockRealmRepository::new();
        realm_repo.expect_get_by_name().returning(move |_| {
            let r = make_realm(realm_id, "test-realm");
            Box::pin(async move { Ok(Some(r)) })
        });

        let mut org_repo = MockOrganizationRepository::new();
        org_repo
            .expect_get_organization_by_id()
            .return_once(move |_| Box::pin(async move { Ok(Some(org)) }));

        let mut attr_repo = MockOrganizationAttributeRepository::new();
        attr_repo
            .expect_list_attributes()
            .return_once(|_| Box::pin(async { Ok(vec![]) }));

        let mut user_repo = MockUserRepository::new();
        user_repo.expect_get_by_id().returning(move |_| {
            let u = user.clone();
            Box::pin(async move { Ok(u) })
        });

        let mut user_role_repo = MockUserRoleRepository::new();
        user_role_repo.expect_get_user_roles().returning(move |_| {
            let role = make_role_with_permission(realm_id, "view_users");
            Box::pin(async move { Ok(vec![role]) })
        });

        let service = build_service(
            realm_repo,
            user_repo,
            user_role_repo,
            org_repo,
            attr_repo,
            MockOrganizationMemberRepository::new(),
        );

        let result = service
            .list_attributes(
                identity,
                ListOrganizationAttributesInput {
                    realm_name: "test-realm".to_string(),
                    organization_id: org_id,
                },
            )
            .await;

        assert!(matches!(result, Ok(v) if v.is_empty()));
    }

    #[tokio::test]
    async fn upsert_attribute_rejects_empty_key() {
        let realm_id = RealmId::new(Uuid::new_v4());
        let realm = make_realm(realm_id, "test-realm");
        let user = make_user(&realm);
        let identity = Identity::User(user.clone());
        let org = make_organization(realm_id);
        let org_id = org.id;

        let mut realm_repo = MockRealmRepository::new();
        realm_repo.expect_get_by_name().returning(move |_| {
            let r = make_realm(realm_id, "test-realm");
            Box::pin(async move { Ok(Some(r)) })
        });

        let mut org_repo = MockOrganizationRepository::new();
        org_repo
            .expect_get_organization_by_id()
            .return_once(move |_| Box::pin(async move { Ok(Some(org)) }));

        let mut user_repo = MockUserRepository::new();
        user_repo.expect_get_by_id().returning(move |_| {
            let u = user.clone();
            Box::pin(async move { Ok(u) })
        });

        let mut user_role_repo = MockUserRoleRepository::new();
        user_role_repo.expect_get_user_roles().returning(move |_| {
            let role = make_role_with_permission(realm_id, "manage_realm");
            Box::pin(async move { Ok(vec![role]) })
        });

        let service = build_service(
            realm_repo,
            user_repo,
            user_role_repo,
            org_repo,
            MockOrganizationAttributeRepository::new(),
            MockOrganizationMemberRepository::new(),
        );

        let result = service
            .upsert_attribute(
                identity,
                UpsertOrganizationAttributeInput {
                    realm_name: "test-realm".to_string(),
                    organization_id: org_id,
                    key: "  ".to_string(),
                    value: "some-value".to_string(),
                },
            )
            .await;

        assert!(matches!(result, Err(CoreError::Invalid)));
    }

    #[tokio::test]
    async fn update_organization_rejects_duplicate_alias() {
        let realm_id = RealmId::new(Uuid::new_v4());
        let realm = make_realm(realm_id, "test-realm");
        let user = make_user(&realm);
        let identity = Identity::User(user.clone());
        let org = make_organization(realm_id);
        let org_id = org.id;

        let mut realm_repo = MockRealmRepository::new();
        realm_repo.expect_get_by_name().returning(move |_| {
            let r = make_realm(realm_id, "test-realm");
            Box::pin(async move { Ok(Some(r)) })
        });

        let mut org_repo = MockOrganizationRepository::new();
        org_repo
            .expect_get_organization_by_id()
            .return_once(move |_| Box::pin(async move { Ok(Some(org)) }));
        org_repo
            .expect_exists_organization_by_realm_and_alias()
            .return_once(|_, _| Box::pin(async { Ok(true) }));

        let mut user_repo = MockUserRepository::new();
        user_repo.expect_get_by_id().returning(move |_| {
            let u = user.clone();
            Box::pin(async move { Ok(u) })
        });

        let mut user_role_repo = MockUserRoleRepository::new();
        user_role_repo.expect_get_user_roles().returning(move |_| {
            let role = make_role_with_permission(realm_id, "manage_realm");
            Box::pin(async move { Ok(vec![role]) })
        });

        let service = build_service(
            realm_repo,
            user_repo,
            user_role_repo,
            org_repo,
            MockOrganizationAttributeRepository::new(),
            MockOrganizationMemberRepository::new(),
        );

        let result = service
            .update_organization(
                identity,
                UpdateOrganizationInput {
                    realm_name: "test-realm".to_string(),
                    organization_id: org_id,
                    name: None,
                    alias: Some("other-org".to_string()),
                    domain: None,
                    redirect_url: None,
                    description: None,
                    enabled: None,
                },
            )
            .await;

        assert!(matches!(result, Err(CoreError::AlreadyExists)));
    }

    // ─── membership tests ─────────────────────────────────────────────────────────

    #[tokio::test]
    async fn add_member_succeeds_for_same_realm_user() {
        let realm_id = RealmId::new(Uuid::new_v4());
        let realm = make_realm(realm_id, "test-realm");
        let admin = make_user(&realm);
        let member_user = make_user(&realm);
        let member_user_id = member_user.id;
        let identity = Identity::User(admin.clone());
        let org = make_organization(realm_id);
        let org_id = org.id;
        let expected_member = make_member(org_id, member_user_id);

        let mut realm_repo = MockRealmRepository::new();
        realm_repo.expect_get_by_name().returning(move |_| {
            let r = make_realm(realm_id, "test-realm");
            Box::pin(async move { Ok(Some(r)) })
        });

        let mut org_repo = MockOrganizationRepository::new();
        org_repo
            .expect_get_organization_by_id()
            .return_once(move |_| Box::pin(async move { Ok(Some(org)) }));

        let mut member_repo = MockOrganizationMemberRepository::new();
        member_repo
            .expect_get_member()
            .return_once(|_, _| Box::pin(async { Ok(None) }));
        member_repo
            .expect_add_member()
            .return_once(move |_, _| Box::pin(async move { Ok(expected_member) }));

        // Policy lookup (admin user)
        let mut user_repo = MockUserRepository::new();
        user_repo.expect_get_by_id().returning(move |id| {
            let admin_clone = admin.clone();
            let member_clone = member_user.clone();
            Box::pin(async move {
                if id == admin_clone.id {
                    Ok(admin_clone)
                } else {
                    Ok(member_clone)
                }
            })
        });

        let mut user_role_repo = MockUserRoleRepository::new();
        user_role_repo.expect_get_user_roles().returning(move |_| {
            let role = make_role_with_permission(realm_id, "manage_users");
            Box::pin(async move { Ok(vec![role]) })
        });

        let service = build_service(
            realm_repo,
            user_repo,
            user_role_repo,
            org_repo,
            MockOrganizationAttributeRepository::new(),
            member_repo,
        );

        let result = service
            .add_member(
                identity,
                AddOrganizationMemberInput {
                    realm_name: "test-realm".to_string(),
                    organization_id: org_id,
                    user_id: member_user_id,
                },
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn add_member_rejects_disabled_organization() {
        let realm_id = RealmId::new(Uuid::new_v4());
        let realm = make_realm(realm_id, "test-realm");
        let admin = make_user(&realm);
        let member_user_id = Uuid::new_v4();
        let identity = Identity::User(admin.clone());
        let mut org = make_organization(realm_id);
        org.enabled = false;
        let org_id = org.id;

        let mut realm_repo = MockRealmRepository::new();
        realm_repo.expect_get_by_name().returning(move |_| {
            let r = make_realm(realm_id, "test-realm");
            Box::pin(async move { Ok(Some(r)) })
        });

        let mut org_repo = MockOrganizationRepository::new();
        org_repo
            .expect_get_organization_by_id()
            .return_once(move |_| Box::pin(async move { Ok(Some(org)) }));

        let mut user_repo = MockUserRepository::new();
        user_repo.expect_get_by_id().returning(move |_| {
            let u = admin.clone();
            Box::pin(async move { Ok(u) })
        });

        let mut user_role_repo = MockUserRoleRepository::new();
        user_role_repo.expect_get_user_roles().returning(move |_| {
            let role = make_role_with_permission(realm_id, "manage_users");
            Box::pin(async move { Ok(vec![role]) })
        });

        let service = build_service(
            realm_repo,
            user_repo,
            user_role_repo,
            org_repo,
            MockOrganizationAttributeRepository::new(),
            MockOrganizationMemberRepository::new(),
        );

        let result = service
            .add_member(
                identity,
                AddOrganizationMemberInput {
                    realm_name: "test-realm".to_string(),
                    organization_id: org_id,
                    user_id: member_user_id,
                },
            )
            .await;

        assert!(matches!(result, Err(CoreError::Invalid)));
    }

    #[tokio::test]
    async fn add_member_rejects_cross_realm_user() {
        let realm_id = RealmId::new(Uuid::new_v4());
        let other_realm_id = RealmId::new(Uuid::new_v4());
        let realm = make_realm(realm_id, "test-realm");
        let other_realm = make_realm(other_realm_id, "other-realm");
        let admin = make_user(&realm);
        let cross_realm_user = make_user(&other_realm); // user from different realm
        let cross_realm_user_id = cross_realm_user.id;
        let identity = Identity::User(admin.clone());
        let org = make_organization(realm_id);
        let org_id = org.id;

        let mut realm_repo = MockRealmRepository::new();
        realm_repo.expect_get_by_name().returning(move |_| {
            let r = make_realm(realm_id, "test-realm");
            Box::pin(async move { Ok(Some(r)) })
        });

        let mut org_repo = MockOrganizationRepository::new();
        org_repo
            .expect_get_organization_by_id()
            .return_once(move |_| Box::pin(async move { Ok(Some(org)) }));

        // Policy call returns admin, member lookup returns cross-realm user
        let mut user_repo = MockUserRepository::new();
        user_repo.expect_get_by_id().returning(move |id| {
            let admin_clone = admin.clone();
            let cross_clone = cross_realm_user.clone();
            Box::pin(async move {
                if id == admin_clone.id {
                    Ok(admin_clone)
                } else {
                    Ok(cross_clone)
                }
            })
        });

        let mut user_role_repo = MockUserRoleRepository::new();
        user_role_repo.expect_get_user_roles().returning(move |_| {
            let role = make_role_with_permission(realm_id, "manage_users");
            Box::pin(async move { Ok(vec![role]) })
        });

        let service = build_service(
            realm_repo,
            user_repo,
            user_role_repo,
            org_repo,
            MockOrganizationAttributeRepository::new(),
            MockOrganizationMemberRepository::new(),
        );

        let result = service
            .add_member(
                identity,
                AddOrganizationMemberInput {
                    realm_name: "test-realm".to_string(),
                    organization_id: org_id,
                    user_id: cross_realm_user_id,
                },
            )
            .await;

        assert!(matches!(result, Err(CoreError::Invalid)));
    }

    #[tokio::test]
    async fn add_member_rejects_duplicate_membership() {
        let realm_id = RealmId::new(Uuid::new_v4());
        let realm = make_realm(realm_id, "test-realm");
        let admin = make_user(&realm);
        let member_user = make_user(&realm);
        let member_user_id = member_user.id;
        let identity = Identity::User(admin.clone());
        let org = make_organization(realm_id);
        let org_id = org.id;
        let existing_member = make_member(org_id, member_user_id);

        let mut realm_repo = MockRealmRepository::new();
        realm_repo.expect_get_by_name().returning(move |_| {
            let r = make_realm(realm_id, "test-realm");
            Box::pin(async move { Ok(Some(r)) })
        });

        let mut org_repo = MockOrganizationRepository::new();
        org_repo
            .expect_get_organization_by_id()
            .return_once(move |_| Box::pin(async move { Ok(Some(org)) }));

        let mut member_repo = MockOrganizationMemberRepository::new();
        member_repo
            .expect_get_member()
            .return_once(move |_, _| Box::pin(async move { Ok(Some(existing_member)) }));

        let mut user_repo = MockUserRepository::new();
        user_repo.expect_get_by_id().returning(move |id| {
            let admin_clone = admin.clone();
            let member_clone = member_user.clone();
            Box::pin(async move {
                if id == admin_clone.id {
                    Ok(admin_clone)
                } else {
                    Ok(member_clone)
                }
            })
        });

        let mut user_role_repo = MockUserRoleRepository::new();
        user_role_repo.expect_get_user_roles().returning(move |_| {
            let role = make_role_with_permission(realm_id, "manage_users");
            Box::pin(async move { Ok(vec![role]) })
        });

        let service = build_service(
            realm_repo,
            user_repo,
            user_role_repo,
            org_repo,
            MockOrganizationAttributeRepository::new(),
            member_repo,
        );

        let result = service
            .add_member(
                identity,
                AddOrganizationMemberInput {
                    realm_name: "test-realm".to_string(),
                    organization_id: org_id,
                    user_id: member_user_id,
                },
            )
            .await;

        assert!(matches!(result, Err(CoreError::AlreadyExists)));
    }

    #[tokio::test]
    async fn remove_member_returns_not_found_when_membership_missing() {
        let realm_id = RealmId::new(Uuid::new_v4());
        let realm = make_realm(realm_id, "test-realm");
        let admin = make_user(&realm);
        let identity = Identity::User(admin.clone());
        let org = make_organization(realm_id);
        let org_id = org.id;

        let mut realm_repo = MockRealmRepository::new();
        realm_repo.expect_get_by_name().returning(move |_| {
            let r = make_realm(realm_id, "test-realm");
            Box::pin(async move { Ok(Some(r)) })
        });

        let mut org_repo = MockOrganizationRepository::new();
        org_repo
            .expect_get_organization_by_id()
            .return_once(move |_| Box::pin(async move { Ok(Some(org)) }));

        let mut member_repo = MockOrganizationMemberRepository::new();
        member_repo
            .expect_get_member()
            .return_once(|_, _| Box::pin(async { Ok(None) }));

        let mut user_repo = MockUserRepository::new();
        user_repo.expect_get_by_id().returning(move |_| {
            let u = admin.clone();
            Box::pin(async move { Ok(u) })
        });

        let mut user_role_repo = MockUserRoleRepository::new();
        user_role_repo.expect_get_user_roles().returning(move |_| {
            let role = make_role_with_permission(realm_id, "manage_users");
            Box::pin(async move { Ok(vec![role]) })
        });

        let service = build_service(
            realm_repo,
            user_repo,
            user_role_repo,
            org_repo,
            MockOrganizationAttributeRepository::new(),
            member_repo,
        );

        let result = service
            .remove_member(
                identity,
                RemoveOrganizationMemberInput {
                    realm_name: "test-realm".to_string(),
                    organization_id: org_id,
                    user_id: Uuid::new_v4(),
                },
            )
            .await;

        assert!(matches!(result, Err(CoreError::NotFound)));
    }

    #[tokio::test]
    async fn list_members_returns_members_for_org() {
        let realm_id = RealmId::new(Uuid::new_v4());
        let realm = make_realm(realm_id, "test-realm");
        let admin = make_user(&realm);
        let identity = Identity::User(admin.clone());
        let org = make_organization(realm_id);
        let org_id = org.id;
        let member = make_member(org_id, Uuid::new_v4());

        let mut realm_repo = MockRealmRepository::new();
        realm_repo.expect_get_by_name().returning(move |_| {
            let r = make_realm(realm_id, "test-realm");
            Box::pin(async move { Ok(Some(r)) })
        });

        let mut org_repo = MockOrganizationRepository::new();
        org_repo
            .expect_get_organization_by_id()
            .return_once(move |_| Box::pin(async move { Ok(Some(org)) }));

        let mut member_repo = MockOrganizationMemberRepository::new();
        member_repo
            .expect_list_members()
            .return_once(move |_| Box::pin(async move { Ok(vec![member]) }));

        let mut user_repo = MockUserRepository::new();
        user_repo.expect_get_by_id().returning(move |_| {
            let u = admin.clone();
            Box::pin(async move { Ok(u) })
        });

        let mut user_role_repo = MockUserRoleRepository::new();
        user_role_repo.expect_get_user_roles().returning(move |_| {
            let role = make_role_with_permission(realm_id, "view_users");
            Box::pin(async move { Ok(vec![role]) })
        });

        let service = build_service(
            realm_repo,
            user_repo,
            user_role_repo,
            org_repo,
            MockOrganizationAttributeRepository::new(),
            member_repo,
        );

        let result = service
            .list_members(
                identity,
                ListOrganizationMembersInput {
                    realm_name: "test-realm".to_string(),
                    organization_id: org_id,
                },
            )
            .await;

        assert!(matches!(result, Ok(v) if v.len() == 1));
    }

    #[tokio::test]
    async fn list_user_organizations_returns_memberships() {
        let realm_id = RealmId::new(Uuid::new_v4());
        let realm = make_realm(realm_id, "test-realm");
        let admin = make_user(&realm);
        let target_user_id = Uuid::new_v4();
        let identity = Identity::User(admin.clone());
        let org_id = OrganizationId::new(Uuid::new_v4());
        let member = make_member(org_id, target_user_id);

        let mut realm_repo = MockRealmRepository::new();
        realm_repo.expect_get_by_name().returning(move |_| {
            let r = make_realm(realm_id, "test-realm");
            Box::pin(async move { Ok(Some(r)) })
        });

        let mut member_repo = MockOrganizationMemberRepository::new();
        member_repo
            .expect_list_organizations_for_user()
            .return_once(move |_| Box::pin(async move { Ok(vec![member]) }));

        let mut user_repo = MockUserRepository::new();
        user_repo.expect_get_by_id().returning(move |_| {
            let u = admin.clone();
            Box::pin(async move { Ok(u) })
        });

        let mut user_role_repo = MockUserRoleRepository::new();
        user_role_repo.expect_get_user_roles().returning(move |_| {
            let role = make_role_with_permission(realm_id, "manage_users");
            Box::pin(async move { Ok(vec![role]) })
        });

        let service = build_service(
            realm_repo,
            user_repo,
            user_role_repo,
            MockOrganizationRepository::new(),
            MockOrganizationAttributeRepository::new(),
            member_repo,
        );

        let result = service
            .list_user_organizations(
                identity,
                ListUserOrganizationsInput {
                    realm_name: "test-realm".to_string(),
                    user_id: target_user_id,
                },
            )
            .await;

        assert!(matches!(result, Ok(v) if v.len() == 1));
    }
}
