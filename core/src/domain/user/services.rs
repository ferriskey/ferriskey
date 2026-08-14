use std::sync::Arc;

use ferriskey_domain::user::commands::BulkDeleteUsersInput;
use tracing::{error, warn};
use uuid::Uuid;

use crate::domain::{
    authentication::value_objects::Identity,
    client::ports::ClientRepository,
    common::{
        entities::app_errors::CoreError,
        policies::{FerriskeyPolicy, Policy, ensure_policy},
    },
    credential::ports::CredentialRepository,
    crypto::HasherRepository,
    password_policy::{
        entity::PasswordPolicy, repository::PasswordPolicyRepository,
        service::violations_to_core_error, validator,
    },
    realm::{entities::Realm, ports::RealmRepository},
    role::{
        entities::{Role, permission::Permissions},
        ports::RoleRepository,
    },
    seawatch::{EventStatus, SecurityEvent, SecurityEventRepository, SecurityEventType},
    user::{
        entities::{
            AssignRoleInput, CreateUserInput, DeleteUserAttributeInput, GetUserAttributesInput,
            GetUserInput, GetUserPermissionsInput, RequiredAction, ResetPasswordInput,
            SetUserAttributesInput, UnassignRoleInput, UpdateUserInput, User, UserAttribute,
        },
        ports::{
            UserAttributeRepository, UserPolicy, UserRepository, UserRequiredActionRepository,
            UserRoleRepository, UserService,
        },
        value_objects::{CreateUserRequest, UpdateUserRequest},
    },
    webhook::{
        entities::{webhook_payload::WebhookPayload, webhook_trigger::WebhookTrigger},
        ports::WebhookRepository,
    },
};
use serde_json::json;

fn normalize_optional_email(email: Option<String>) -> Option<String> {
    email.and_then(|e| {
        let trimmed = e.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

#[derive(Clone, Debug)]
pub struct UserServiceImpl<R, U, C, UR, CR, H, RO, URA, W, SE, UAR, PPR>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    CR: CredentialRepository,
    H: HasherRepository,
    RO: RoleRepository,
    URA: UserRequiredActionRepository,
    W: WebhookRepository,
    SE: SecurityEventRepository,
    UAR: UserAttributeRepository,
    PPR: PasswordPolicyRepository,
{
    pub(crate) realm_repository: Arc<R>,
    pub(crate) user_repository: Arc<U>,
    pub(crate) credential_repository: Arc<CR>,
    pub(crate) hasher_repository: Arc<H>,
    pub(crate) user_role_repository: Arc<UR>,
    pub(crate) role_repository: Arc<RO>,
    pub(crate) user_required_action_repository: Arc<URA>,
    pub(crate) user_attribute_repository: Arc<UAR>,
    pub(crate) webhook_repository: Arc<W>,
    pub(crate) security_event_repository: Arc<SE>,
    pub(crate) password_policy_repository: Arc<PPR>,

    pub(crate) policy: Arc<FerriskeyPolicy<U, C, UR>>,
}

impl<R, U, C, UR, CR, H, RO, URA, W, SE, UAR, PPR>
    UserServiceImpl<R, U, C, UR, CR, H, RO, URA, W, SE, UAR, PPR>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    CR: CredentialRepository,
    H: HasherRepository,
    RO: RoleRepository,
    URA: UserRequiredActionRepository,
    W: WebhookRepository,
    SE: SecurityEventRepository,
    UAR: UserAttributeRepository,
    PPR: PasswordPolicyRepository,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        realm_repository: Arc<R>,
        user_repository: Arc<U>,
        credential_repository: Arc<CR>,
        hasher_repository: Arc<H>,
        user_role_repository: Arc<UR>,
        role_repository: Arc<RO>,
        user_required_action_repository: Arc<URA>,
        user_attribute_repository: Arc<UAR>,
        webhook_repository: Arc<W>,
        security_event_repository: Arc<SE>,
        password_policy_repository: Arc<PPR>,
        policy: Arc<FerriskeyPolicy<U, C, UR>>,
    ) -> Self {
        Self {
            realm_repository,
            user_repository,
            credential_repository,
            hasher_repository,
            user_role_repository,
            role_repository,
            user_required_action_repository,
            user_attribute_repository,
            webhook_repository,
            security_event_repository,
            password_policy_repository,
            policy,
        }
    }

    /// Load a user, refusing to look outside `realm` (FK-004).
    ///
    /// Authorization is decided against the realm named in the URL, but the target
    /// used to be fetched by bare UUID. A tenant administrator holding `ManageUsers`
    /// on their own realm could therefore reset the password of, delete, or read any
    /// account of any other realm — the master administrator included — provided they
    /// knew its identifier.
    ///
    /// `NotFound` rather than `Forbidden`: telling a caller that an id exists but
    /// belongs elsewhere is itself a cross-tenant disclosure.
    ///
    /// Cross-realm access *from* `master` stays legitimate and is decided upstream by
    /// the policy layer (`can_access_realm`); this only binds the object to the realm
    /// the request addressed.
    async fn load_user_in_realm(&self, user_id: Uuid, realm: &Realm) -> Result<User, CoreError> {
        let user = self.user_repository.get_by_id(user_id).await?;

        if user.realm_id != realm.id {
            warn!(
                user_id = %user_id,
                user_realm_id = %Uuid::from(user.realm_id),
                request_realm_id = %Uuid::from(realm.id),
                "Refused cross-realm access to a user"
            );
            return Err(CoreError::NotFound);
        }

        Ok(user)
    }

    /// Same binding for roles: granting a role of another realm would carry its
    /// permissions across the tenant boundary.
    async fn load_role_in_realm(&self, role_id: Uuid, realm: &Realm) -> Result<Role, CoreError> {
        let role = self
            .role_repository
            .get_by_id(role_id)
            .await?
            .ok_or_else(|| {
                warn!(role_id = %role_id, "Role not found");
                CoreError::NotFound
            })?;

        if role.realm_id != realm.id {
            warn!(
                role_id = %role_id,
                role_realm_id = %Uuid::from(role.realm_id),
                request_realm_id = %Uuid::from(realm.id),
                "Refused cross-realm access to a role"
            );
            return Err(CoreError::NotFound);
        }

        Ok(role)
    }
}

impl<R, U, C, UR, CR, H, RO, URA, W, SE, UAR, PPR> UserService
    for UserServiceImpl<R, U, C, UR, CR, H, RO, URA, W, SE, UAR, PPR>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    CR: CredentialRepository,
    H: HasherRepository,
    RO: RoleRepository,
    URA: UserRequiredActionRepository,
    W: WebhookRepository,
    SE: SecurityEventRepository,
    UAR: UserAttributeRepository,
    PPR: PasswordPolicyRepository,
{
    async fn delete_user(
        &self,
        identity: Identity,
        realm_name: String,
        user_id: Uuid,
    ) -> Result<u64, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_id = realm.id;
        ensure_policy(
            self.policy.can_update_user(&identity, &realm).await,
            "insufficient permissions",
        )?;

        let user = self.load_user_in_realm(user_id, &realm).await?;

        let count = self
            .user_repository
            .delete_user(user_id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        self.security_event_repository
            .store_event(
                SecurityEvent::new(
                    realm_id,
                    SecurityEventType::UserDeleted,
                    EventStatus::Success,
                    identity.id(),
                )
                .with_target("user".to_string(), user.id, None),
            )
            .await?;

        self.webhook_repository
            .notify(
                realm_id,
                WebhookPayload::new(WebhookTrigger::UserDeleted, realm_id.into(), Some(user)),
            )
            .await?;

        Ok(count)
    }

    async fn reset_password(
        &self,
        identity: Identity,
        input: ResetPasswordInput,
    ) -> Result<(), CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await
            .map_err(|e| {
                error!(
                    "reset_password: failed to fetch realm {}: {e:?}",
                    input.realm_name
                );
                CoreError::InvalidRealm
            })?
            .ok_or_else(|| {
                warn!("reset_password: realm {} not found", input.realm_name);
                CoreError::InvalidRealm
            })?;

        ensure_policy(
            self.policy.can_update_user(&identity, &realm).await,
            "insufficient permissions",
        )?;

        let policy = self
            .password_policy_repository
            .find_by_realm_id(realm.id.into())
            .await
            .map_err(|e| {
                error!(
                    "reset_password: failed to load password policy for realm {:?}: {e:?}",
                    realm.id
                );
                e
            })?
            .unwrap_or_else(|| PasswordPolicy::default(realm.id.into()));

        // Loaded before any write, and no longer with `.ok()`: swallowing the error
        // meant a target that could not be read was still reset.
        let target_user = self.load_user_in_realm(input.user_id, &realm).await?;

        let username = target_user.username.clone();
        let email_local_buf = target_user
            .email
            .as_deref()
            .and_then(|e| e.split('@').next())
            .map(str::to_string);
        let (username_ref, email_local_ref) = (Some(username.as_str()), email_local_buf.as_deref());

        validator::validate(&input.password, &policy, username_ref, email_local_ref).map_err(
            |e| {
                warn!(
                    "reset_password: password policy violation for user {}: {e:?}",
                    input.user_id
                );
                violations_to_core_error(e)
            },
        )?;

        let password_credential = self
            .credential_repository
            .get_password_credential(input.user_id)
            .await;

        if password_credential.is_ok() {
            self.credential_repository
                .delete_password_credential(input.user_id)
                .await
                .map_err(|e| {
                    error!(
                        "reset_password: failed to delete existing password credential for user {}: {e:?}",
                        input.user_id
                    );
                    CoreError::DeletePasswordCredentialError
                })?;
        }

        let hash_result = self
            .hasher_repository
            .hash_password(&input.password)
            .await
            .map_err(|e| {
                error!(
                    "reset_password: failed to hash password for user {}: {e:?}",
                    input.user_id
                );
                CoreError::HashPasswordError(e.to_string())
            })?;

        self.credential_repository
            .create_credential(
                input.user_id,
                "password".into(),
                hash_result,
                "".into(),
                input.temporary,
            )
            .await
            .map_err(|e| {
                error!(
                    "reset_password: failed to create password credential for user {}: {e:?}",
                    input.user_id
                );
                CoreError::CreateCredentialError
            })?;

        self.security_event_repository
            .store_event(
                SecurityEvent::new(
                    realm.id,
                    SecurityEventType::PasswordReset,
                    EventStatus::Success,
                    identity.id(),
                )
                .with_target("user".to_string(), input.user_id, None),
            )
            .await
            .map_err(|e| {
                error!(
                    "reset_password: failed to store security event for user {}: {e:?}",
                    input.user_id
                );
                e
            })?;

        // @TODO: webhook call action

        Ok(())
    }

    async fn update_user(
        &self,
        identity: Identity,
        input: UpdateUserInput,
    ) -> Result<User, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_id = realm.id;
        ensure_policy(
            self.policy.can_update_user(&identity, &realm).await,
            "You are not allowed to view users in this realm.",
        )?;

        // Bind the target to the realm before writing: this method used not to load
        // it at all, so an id from another tenant was written straight through.
        self.load_user_in_realm(input.user_id, &realm).await?;

        let user = self
            .user_repository
            .update_user(
                input.user_id,
                UpdateUserRequest {
                    email: normalize_optional_email(input.email),
                    email_verified: input.email_verified.unwrap_or(false),
                    enabled: input.enabled,
                    firstname: input.firstname,
                    lastname: input.lastname,
                    required_actions: None,
                },
            )
            .await?;

        if let Some(required_actions) = input.required_actions {
            self.user_required_action_repository
                .clear_required_actions(user.id)
                .await
                .map_err(|_| CoreError::InternalServerError)?;

            for action in required_actions {
                let required_action: RequiredAction = RequiredAction::try_from(action.clone())
                    .map_err(|_| CoreError::InvalidRequiredAction(action))?;
                self.user_required_action_repository
                    .add_required_action(user.id, required_action)
                    .await
                    .map_err(|_| CoreError::InternalServerError)?;
            }
        }

        self.webhook_repository
            .notify(
                realm_id,
                WebhookPayload::new(
                    WebhookTrigger::UserUpdated,
                    realm_id.into(),
                    Some(user.clone()),
                ),
            )
            .await?;

        Ok(user)
    }

    async fn get_users(
        &self,
        identity: Identity,
        realm_name: String,
    ) -> Result<Vec<User>, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_id = realm.id;

        ensure_policy(
            self.policy.can_view_user(&identity, &realm).await,
            "You are not allowed to view users in this realm.",
        )?;

        self.user_repository
            .find_by_realm_id(realm_id)
            .await
            .map_err(|_| CoreError::InternalServerError)
    }

    async fn assign_role(
        &self,
        identity: Identity,
        input: AssignRoleInput,
    ) -> Result<(), CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_id = realm.id;

        ensure_policy(
            self.policy.can_update_user(&identity, &realm).await,
            "insufficient permissions",
        )?;

        // Both ends are bound: a foreign user must not be granted a role, and a
        // foreign role must not have its permissions carried across the boundary.
        self.load_user_in_realm(input.user_id, &realm).await?;
        let role = self.load_role_in_realm(input.role_id, &realm).await?;

        self.user_role_repository
            .assign_role(input.user_id, input.role_id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        self.security_event_repository
            .store_event(
                SecurityEvent::new(
                    realm_id,
                    SecurityEventType::RoleAssigned,
                    EventStatus::Success,
                    identity.id(),
                )
                .with_target(
                    "user".to_string(),
                    input.user_id,
                    Some(role.name.clone()),
                ),
            )
            .await?;

        self.webhook_repository
            .notify(
                realm_id,
                WebhookPayload::new(
                    WebhookTrigger::UserRoleAssigned,
                    realm_id.into(),
                    Some(role.clone()),
                ),
            )
            .await?;

        Ok(())
    }

    async fn bulk_delete_users(
        &self,
        identity: Identity,
        input: BulkDeleteUsersInput,
    ) -> Result<u64, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_id = realm.id;

        ensure_policy(
            self.policy.can_delete_user(&identity, &realm).await,
            "insufficient permissions",
        )?;

        // Scoped in SQL rather than by pre-checking each id: a `WHERE realm_id = ?`
        // predicate cannot be forgotten by a future caller, and ids from another
        // tenant simply match no row.
        let count = self
            .user_repository
            .bulk_delete_user(realm_id, input.ids.clone())
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        self.security_event_repository
            .store_event(
                SecurityEvent::new(
                    realm_id,
                    SecurityEventType::UserDeleted,
                    EventStatus::Success,
                    identity.id(),
                )
                .with_details(json!({ "user_ids": input.ids })),
            )
            .await?;

        self.webhook_repository
            .notify(
                realm_id,
                WebhookPayload::new(
                    WebhookTrigger::UserBulkDeleted,
                    realm_id.into(),
                    Some(input.ids),
                ),
            )
            .await?;

        Ok(count)
    }

    async fn create_user(
        &self,
        identity: Identity,
        input: CreateUserInput,
    ) -> Result<User, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_id = realm.id;
        ensure_policy(
            self.policy.can_create_user(&identity, &realm).await,
            "insufficient permissions",
        )?;

        let mut user = self
            .user_repository
            .create_user(CreateUserRequest {
                client_id: None,
                realm_id,
                username: input.username,
                firstname: input.firstname,
                lastname: input.lastname,
                email: normalize_optional_email(input.email),
                email_verified: input.email_verified.unwrap_or(false),
                enabled: true,
            })
            .await?;

        user.realm = Some(realm);

        self.security_event_repository
            .store_event(
                SecurityEvent::new(
                    realm_id,
                    SecurityEventType::UserCreated,
                    EventStatus::Success,
                    identity.id(),
                )
                .with_target("user".to_string(), user.id, None),
            )
            .await?;

        self.webhook_repository
            .notify(
                realm_id,
                WebhookPayload::new(
                    WebhookTrigger::UserCreated,
                    realm_id.into(),
                    Some(user.clone()),
                ),
            )
            .await?;

        Ok(user)
    }

    async fn get_user(&self, identity: Identity, input: GetUserInput) -> Result<User, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_view_user(&identity, &realm).await,
            "insufficient permissions",
        )?;

        self.load_user_in_realm(input.user_id, &realm).await
    }

    async fn unassign_role(
        &self,
        identity: Identity,
        input: UnassignRoleInput,
    ) -> Result<(), CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_id = realm.id;
        ensure_policy(
            self.policy.can_update_user(&identity, &realm).await,
            "insufficient permissions",
        )?;

        self.load_user_in_realm(input.user_id, &realm).await?;
        let role = self.load_role_in_realm(input.role_id, &realm).await?;

        self.user_role_repository
            .revoke_role(input.user_id, input.role_id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        self.security_event_repository
            .store_event(
                SecurityEvent::new(
                    realm_id,
                    SecurityEventType::RoleUnassigned,
                    EventStatus::Success,
                    identity.id(),
                )
                .with_target(
                    "user".to_string(),
                    input.user_id,
                    Some(role.name.clone()),
                ),
            )
            .await?;

        self.webhook_repository
            .notify(
                realm_id,
                WebhookPayload::new(
                    WebhookTrigger::UserUpdated,
                    realm_id.into(),
                    Some(role.clone()),
                ),
            )
            .await?;

        Ok(())
    }

    async fn get_user_permissions(
        &self,
        identity: Identity,
        input: GetUserPermissionsInput,
    ) -> Result<Vec<Permissions>, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy
                .can_view_user_permissions(&identity, &realm, input.user_id)
                .await,
            "insufficient permissions",
        )?;

        // This site had its own weaker variant of the check: it compared realm *names*
        // via the optionally-loaded relation, and carried a `|| != "master"` escape
        // hatch that let any tenant read the permissions of any master-realm user.
        let user = self.load_user_in_realm(input.user_id, &realm).await?;

        let permissions = self
            .policy
            .get_permission_for_target_realm(&user, &realm)
            .await?;

        Ok(permissions.into_iter().collect())
    }

    async fn get_user_attributes(
        &self,
        identity: Identity,
        input: GetUserAttributesInput,
    ) -> Result<Vec<UserAttribute>, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_view_user(&identity, &realm).await,
            "insufficient permissions",
        )?;

        let user = self.user_repository.get_by_id(input.user_id).await?;

        if Into::<uuid::Uuid>::into(user.realm_id) != Into::<uuid::Uuid>::into(realm.id) {
            return Err(CoreError::NotFound);
        }

        self.user_attribute_repository
            .list_by_user_id(input.user_id)
            .await
    }

    async fn set_user_attributes(
        &self,
        identity: Identity,
        input: SetUserAttributesInput,
    ) -> Result<Vec<UserAttribute>, CoreError> {
        if input.attributes.keys().any(|key| key.trim().is_empty()) {
            return Err(CoreError::Invalid);
        }

        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_update_user(&identity, &realm).await,
            "insufficient permissions",
        )?;

        let user = self.user_repository.get_by_id(input.user_id).await?;

        if Into::<Uuid>::into(user.realm_id) != Into::<Uuid>::into(realm.id) {
            return Err(CoreError::NotFound);
        }

        self.user_attribute_repository
            .upsert_many(input.user_id, realm.id, input.attributes)
            .await
    }

    async fn delete_user_attribute(
        &self,
        identity: Identity,
        input: DeleteUserAttributeInput,
    ) -> Result<(), CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_update_user(&identity, &realm).await,
            "insufficient permissions",
        )?;

        let user = self.user_repository.get_by_id(input.user_id).await?;

        if Into::<uuid::Uuid>::into(user.realm_id) != Into::<uuid::Uuid>::into(realm.id) {
            return Err(CoreError::NotFound);
        }

        self.user_attribute_repository
            .delete_by_key(input.user_id, input.key)
            .await
    }

    async fn unlock_user(
        &self,
        identity: Identity,
        realm_name: String,
        user_id: Uuid,
    ) -> Result<(), CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_update_user(&identity, &realm).await,
            "insufficient permissions",
        )?;

        let user = self.user_repository.get_by_id(user_id).await?;

        if Into::<uuid::Uuid>::into(user.realm_id) != Into::<uuid::Uuid>::into(realm.id) {
            return Err(CoreError::NotFound);
        }

        self.user_repository.unlock_user(user_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        client::ports::MockClientRepository,
        common::services::tests::{
            create_test_realm_with_name, create_test_user_identity_with_realm,
            create_test_user_with_params_and_realm,
        },
        credential::ports::MockCredentialRepository,
        crypto::MockHasherRepository,
        password_policy::repository::MockPasswordPolicyRepository,
        realm::{entities::Realm, ports::MockRealmRepository},
        role::ports::MockRoleRepository,
        seawatch::ports::MockSecurityEventRepository,
        user::ports::{
            MockUserAttributeRepository, MockUserRepository, MockUserRequiredActionRepository,
            MockUserRoleRepository,
        },
        webhook::{entities::webhook_payload::WebhookPayload, ports::MockWebhookRepository},
    };

    struct UserServiceTestBuilder {
        realm_repo: Arc<MockRealmRepository>,
        user_repo: Arc<MockUserRepository>,
        credential_repo: Arc<MockCredentialRepository>,
        hasher_repo: Arc<MockHasherRepository>,
        user_role_repo: Arc<MockUserRoleRepository>,
        role_repo: Arc<MockRoleRepository>,
        user_required_action_repo: Arc<MockUserRequiredActionRepository>,
        user_attribute_repo: Arc<MockUserAttributeRepository>,
        webhook_repo: Arc<MockWebhookRepository>,
        client_repo: Arc<MockClientRepository>,
        security_event_repo: Arc<MockSecurityEventRepository>,
        password_policy_repo: Arc<MockPasswordPolicyRepository>,
    }

    impl UserServiceTestBuilder {
        pub fn new() -> Self {
            Self {
                realm_repo: Arc::new(MockRealmRepository::new()),
                user_repo: Arc::new(MockUserRepository::new()),
                credential_repo: Arc::new(MockCredentialRepository::new()),
                hasher_repo: Arc::new(MockHasherRepository::new()),
                user_role_repo: Arc::new(MockUserRoleRepository::new()),
                role_repo: Arc::new(MockRoleRepository::new()),
                user_required_action_repo: Arc::new(MockUserRequiredActionRepository::new()),
                user_attribute_repo: Arc::new(MockUserAttributeRepository::new()),
                webhook_repo: Arc::new(MockWebhookRepository::new()),
                client_repo: Arc::new(MockClientRepository::new()),
                security_event_repo: Arc::new(MockSecurityEventRepository::new()),
                password_policy_repo: Arc::new(MockPasswordPolicyRepository::new()),
            }
        }

        fn with_realm(mut self, realm_name: String, realm: Realm) -> Self {
            Arc::get_mut(&mut self.realm_repo)
                .unwrap()
                .expect_get_by_name()
                .with(mockall::predicate::eq(realm_name))
                .times(1)
                .return_once(move |_| Box::pin(async move { Ok(Some(realm)) }));
            self
        }

        fn with_user_permissions(
            mut self,
            user_id: uuid::Uuid,
            roles: Vec<crate::domain::role::entities::Role>,
        ) -> Self {
            Arc::get_mut(&mut self.user_role_repo)
                .unwrap()
                .expect_get_user_roles()
                .with(mockall::predicate::eq(user_id))
                .times(1)
                .return_once(move |_| Box::pin(async move { Ok(roles) }));
            self
        }

        fn with_create_user_success(mut self, created_user: User) -> Self {
            Arc::get_mut(&mut self.user_repo)
                .unwrap()
                .expect_create_user()
                .times(1)
                .return_once(move |_| Box::pin(async move { Ok(created_user) }));
            Arc::get_mut(&mut self.security_event_repo)
                .unwrap()
                .expect_store_event()
                .times(1)
                .return_once(|_| Box::pin(async move { Ok(()) }));
            self
        }

        fn with_create_user_email_exists(mut self) -> Self {
            Arc::get_mut(&mut self.user_repo)
                .unwrap()
                .expect_create_user()
                .times(1)
                .return_once(move |_| Box::pin(async move { Err(CoreError::EmailAlreadyExists) }));
            self
        }

        fn with_update_user_success(mut self, user_id: uuid::Uuid, updated_user: User) -> Self {
            Arc::get_mut(&mut self.user_repo)
                .unwrap()
                .expect_update_user()
                .with(
                    mockall::predicate::eq(user_id),
                    mockall::predicate::always(),
                )
                .times(1)
                .return_once(move |_, _| Box::pin(async move { Ok(updated_user) }));
            self
        }

        fn with_update_user_email_exists(mut self, user_id: uuid::Uuid) -> Self {
            Arc::get_mut(&mut self.user_repo)
                .unwrap()
                .expect_update_user()
                .with(
                    mockall::predicate::eq(user_id),
                    mockall::predicate::always(),
                )
                .times(1)
                .return_once(move |_, _| {
                    Box::pin(async move { Err(CoreError::EmailAlreadyExists) })
                });
            self
        }

        /// Stub the `get_by_id` that `load_role_in_realm` performs.
        fn with_role(mut self, role: crate::domain::role::entities::Role) -> Self {
            Arc::get_mut(&mut self.role_repo)
                .unwrap()
                .expect_get_by_id()
                .with(mockall::predicate::eq(role.id))
                .times(1)
                .return_once(move |_| Box::pin(async move { Ok(Some(role)) }));
            self
        }

        /// Stub the `get_by_id` that `load_user_in_realm` performs before any write.
        fn with_target_user(mut self, user: User) -> Self {
            Arc::get_mut(&mut self.user_repo)
                .unwrap()
                .expect_get_by_id()
                .with(mockall::predicate::eq(user.id))
                .times(1)
                .return_once(move |_| Box::pin(async move { Ok(user) }));
            self
        }

        fn with_webhook_notify(mut self) -> Self {
            Arc::get_mut(&mut self.webhook_repo)
                .unwrap()
                .expect_notify::<User>()
                .times(1)
                .return_once(|_, _: WebhookPayload<User>| Box::pin(async move { Ok(()) }));
            self
        }

        fn build(
            self,
        ) -> UserServiceImpl<
            MockRealmRepository,
            MockUserRepository,
            MockClientRepository,
            MockUserRoleRepository,
            MockCredentialRepository,
            MockHasherRepository,
            MockRoleRepository,
            MockUserRequiredActionRepository,
            MockWebhookRepository,
            MockSecurityEventRepository,
            MockUserAttributeRepository,
            MockPasswordPolicyRepository,
        > {
            use crate::domain::common::policies::FerriskeyPolicy;

            let policy = FerriskeyPolicy::new(
                self.user_repo.clone(),
                self.client_repo.clone(),
                self.user_role_repo.clone(),
            );

            UserServiceImpl::new(
                self.realm_repo,
                self.user_repo,
                self.credential_repo,
                self.hasher_repo,
                self.user_role_repo,
                self.role_repo,
                self.user_required_action_repo,
                self.user_attribute_repo,
                self.webhook_repo,
                self.security_event_repo,
                self.password_policy_repo,
                Arc::new(policy),
            )
        }
    }

    fn create_admin_role(realm: &Realm) -> crate::domain::role::entities::Role {
        crate::domain::role::entities::Role {
            id: uuid::Uuid::new_v4(),
            name: "admin".to_string(),
            description: None,
            permissions: vec![
                crate::domain::role::entities::permission::Permissions::ManageUsers.name(),
            ],
            realm_id: realm.id,
            client_id: None,
            client: None,
            require_mfa: false,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_create_user_with_duplicate_email_in_same_realm_fails() {
        let realm = create_test_realm_with_name("test-realm");
        let identity = create_test_user_identity_with_realm(&realm);
        let admin_role = create_admin_role(&realm);

        let user_id = match &identity {
            Identity::User(u) => u.id,
            _ => panic!("Expected user identity"),
        };

        // Repository returns EmailAlreadyExists when constraint is violated
        let service = UserServiceTestBuilder::new()
            .with_realm("test-realm".to_string(), realm.clone())
            .with_user_permissions(user_id, vec![admin_role])
            .with_create_user_email_exists()
            .build();

        let input = CreateUserInput {
            realm_name: "test-realm".to_string(),
            username: "new_user".to_string(),
            firstname: Some("New".to_string()),
            lastname: Some("User".to_string()),
            email: Some("test@example.com".to_string()),
            email_verified: Some(false),
        };

        let result = service.create_user(identity, input).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CoreError::EmailAlreadyExists));
    }

    #[tokio::test]
    async fn test_create_user_with_unique_email_succeeds() {
        let realm = create_test_realm_with_name("test-realm");
        let identity = create_test_user_identity_with_realm(&realm);
        let admin_role = create_admin_role(&realm);

        let user_id = match &identity {
            Identity::User(u) => u.id,
            _ => panic!("Expected user identity"),
        };

        let new_user = create_test_user_with_params_and_realm(
            &realm,
            "new_user",
            "unique@example.com".to_string(),
            true,
        );

        let service = UserServiceTestBuilder::new()
            .with_realm("test-realm".to_string(), realm.clone())
            .with_user_permissions(user_id, vec![admin_role])
            .with_create_user_success(new_user.clone())
            .with_webhook_notify()
            .build();

        let input = CreateUserInput {
            realm_name: "test-realm".to_string(),
            username: "new_user".to_string(),
            firstname: Some("New".to_string()),
            lastname: Some("User".to_string()),
            email: Some("unique@example.com".to_string()),
            email_verified: Some(false),
        };

        let result = service.create_user(identity, input).await;

        assert!(result.is_ok());
        let created_user = result.unwrap();
        assert_eq!(created_user.email, Some("unique@example.com".to_string()));
    }

    #[tokio::test]
    async fn test_update_user_with_another_users_email_fails() {
        let realm = create_test_realm_with_name("test-realm");
        let identity = create_test_user_identity_with_realm(&realm);
        let admin_role = create_admin_role(&realm);

        let user_id = match &identity {
            Identity::User(u) => u.id,
            _ => panic!("Expected user identity"),
        };

        let user_to_update = create_test_user_with_params_and_realm(
            &realm,
            "user_to_update",
            "original@example.com".to_string(),
            true,
        );

        // Repository returns EmailAlreadyExists when constraint is violated
        let service = UserServiceTestBuilder::new()
            .with_realm("test-realm".to_string(), realm.clone())
            .with_user_permissions(user_id, vec![admin_role])
            .with_target_user(user_to_update.clone())
            .with_update_user_email_exists(user_to_update.id)
            .build();

        let input = UpdateUserInput {
            realm_name: "test-realm".to_string(),
            user_id: user_to_update.id,
            firstname: Some("Updated".to_string()),
            lastname: Some("User".to_string()),
            email: Some("taken@example.com".to_string()), // Email belongs to another user
            email_verified: Some(true),
            enabled: true,
            required_actions: None,
        };

        let result = service.update_user(identity, input).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CoreError::EmailAlreadyExists));
    }

    #[tokio::test]
    async fn test_update_user_keeping_own_email_succeeds() {
        let realm = create_test_realm_with_name("test-realm");
        let identity = create_test_user_identity_with_realm(&realm);
        let admin_role = create_admin_role(&realm);

        let user_id = match &identity {
            Identity::User(u) => u.id,
            _ => panic!("Expected user identity"),
        };

        let mut user_to_update = create_test_user_with_params_and_realm(
            &realm,
            "user_to_update",
            "myemail@example.com".to_string(),
            true,
        );

        let update_user_id = user_to_update.id;
        user_to_update.firstname = Some("Updated".to_string());

        // Keeping own email doesn't violate the constraint
        let service = UserServiceTestBuilder::new()
            .with_realm("test-realm".to_string(), realm.clone())
            .with_user_permissions(user_id, vec![admin_role])
            .with_target_user(user_to_update.clone())
            .with_update_user_success(update_user_id, user_to_update.clone())
            .with_webhook_notify()
            .build();

        let input = UpdateUserInput {
            realm_name: "test-realm".to_string(),
            user_id: update_user_id,
            firstname: Some("Updated".to_string()),
            lastname: Some("User".to_string()),
            email: Some("myemail@example.com".to_string()), // Same email as before
            email_verified: Some(true),
            enabled: true,
            required_actions: None,
        };

        let result = service.update_user(identity, input).await;

        assert!(result.is_ok());
        let updated_user = result.unwrap();
        assert_eq!(updated_user.email, Some("myemail@example.com".to_string()));
        assert_eq!(updated_user.firstname, Some("Updated".to_string()));
    }

    #[tokio::test]
    async fn test_update_user_with_new_unique_email_succeeds() {
        let realm = create_test_realm_with_name("test-realm");
        let identity = create_test_user_identity_with_realm(&realm);
        let admin_role = create_admin_role(&realm);

        let user_id = match &identity {
            Identity::User(u) => u.id,
            _ => panic!("Expected user identity"),
        };

        let mut user_to_update = create_test_user_with_params_and_realm(
            &realm,
            "user_to_update",
            "old@example.com".to_string(),
            true,
        );

        let update_user_id = user_to_update.id;
        user_to_update.email = Some("newemail@example.com".to_string());

        let service = UserServiceTestBuilder::new()
            .with_realm("test-realm".to_string(), realm.clone())
            .with_user_permissions(user_id, vec![admin_role])
            .with_target_user(user_to_update.clone())
            .with_update_user_success(update_user_id, user_to_update.clone())
            .with_webhook_notify()
            .build();

        let input = UpdateUserInput {
            realm_name: "test-realm".to_string(),
            user_id: update_user_id,
            firstname: Some("Test".to_string()),
            lastname: Some("User".to_string()),
            email: Some("newemail@example.com".to_string()), // New unique email
            email_verified: Some(true),
            enabled: true,
            required_actions: None,
        };

        let result = service.update_user(identity, input).await;

        assert!(result.is_ok());
        let updated_user = result.unwrap();
        assert_eq!(updated_user.email, Some("newemail@example.com".to_string()));
    }

    // ---- FK-004: the realm in the URL binds the target -----------------------
    //
    // Each of these builds a service that has *no* expectation for the mutating
    // repository call. If the boundary check regressed, the write would fire and
    // mockall would panic on an unexpected call — so the assertion is doubled.

    #[tokio::test]
    async fn update_user_refuses_a_target_from_another_realm() {
        let attacker_realm = create_test_realm_with_name("tenant-a");
        let victim_realm = create_test_realm_with_name("tenant-b");
        let identity = create_test_user_identity_with_realm(&attacker_realm);
        let admin_role = create_admin_role(&attacker_realm);

        let admin_id = match &identity {
            Identity::User(u) => u.id,
            _ => panic!("Expected user identity"),
        };

        let victim = create_test_user_with_params_and_realm(
            &victim_realm,
            "victim",
            "victim@tenant-b.example".to_string(),
            true,
        );

        let service = UserServiceTestBuilder::new()
            .with_realm("tenant-a".to_string(), attacker_realm.clone())
            .with_user_permissions(admin_id, vec![admin_role])
            .with_target_user(victim.clone())
            .build();

        let result = service
            .update_user(
                identity,
                UpdateUserInput {
                    realm_name: "tenant-a".to_string(),
                    user_id: victim.id,
                    firstname: Some("Pwned".to_string()),
                    lastname: None,
                    email: Some("attacker@evil.example".to_string()),
                    email_verified: Some(true),
                    enabled: true,
                    required_actions: None,
                },
            )
            .await;

        assert!(matches!(result, Err(CoreError::NotFound)));
    }

    #[tokio::test]
    async fn get_user_refuses_a_target_from_another_realm() {
        let attacker_realm = create_test_realm_with_name("tenant-a");
        let victim_realm = create_test_realm_with_name("tenant-b");
        let identity = create_test_user_identity_with_realm(&attacker_realm);
        // `can_view_user` wants ViewUsers or ManageRealm, which the shared admin
        // helper does not grant — without it the policy would reject first and the
        // test would pass without ever reaching the realm binding.
        let mut viewer_role = create_admin_role(&attacker_realm);
        viewer_role.permissions.push(Permissions::ViewUsers.name());

        let admin_id = match &identity {
            Identity::User(u) => u.id,
            _ => panic!("Expected user identity"),
        };

        let victim = create_test_user_with_params_and_realm(
            &victim_realm,
            "victim",
            "victim@tenant-b.example".to_string(),
            true,
        );

        let service = UserServiceTestBuilder::new()
            .with_realm("tenant-a".to_string(), attacker_realm.clone())
            .with_user_permissions(admin_id, vec![viewer_role])
            .with_target_user(victim.clone())
            .build();

        let result = service
            .get_user(
                identity,
                GetUserInput {
                    realm_name: "tenant-a".to_string(),
                    user_id: victim.id,
                },
            )
            .await;

        assert!(
            matches!(result, Err(CoreError::NotFound)),
            "reading a user of another realm must not disclose it"
        );
    }

    #[tokio::test]
    async fn assign_role_refuses_a_role_from_another_realm() {
        // The user is legitimate; the *role* is not. Granting it would carry another
        // tenant's permissions across the boundary.
        let realm = create_test_realm_with_name("tenant-a");
        let other_realm = create_test_realm_with_name("tenant-b");
        let identity = create_test_user_identity_with_realm(&realm);
        let admin_role = create_admin_role(&realm);

        let admin_id = match &identity {
            Identity::User(u) => u.id,
            _ => panic!("Expected user identity"),
        };

        let target = create_test_user_with_params_and_realm(
            &realm,
            "member",
            "member@tenant-a.example".to_string(),
            true,
        );
        let foreign_role = create_admin_role(&other_realm);

        let service = UserServiceTestBuilder::new()
            .with_realm("tenant-a".to_string(), realm.clone())
            .with_user_permissions(admin_id, vec![admin_role])
            .with_target_user(target.clone())
            .with_role(foreign_role.clone())
            .build();

        let result = service
            .assign_role(
                identity,
                AssignRoleInput {
                    realm_name: "tenant-a".to_string(),
                    user_id: target.id,
                    role_id: foreign_role.id,
                },
            )
            .await;

        assert!(matches!(result, Err(CoreError::NotFound)));
    }
}
