use std::sync::Arc;

use uuid::Uuid;

use crate::domain::{
    authentication::value_objects::Identity,
    client::ports::ClientRepository,
    common::{
        entities::app_errors::CoreError,
        policies::{FerriskeyPolicy, ensure_policy},
    },
    realm::entities::Realm,
    user::ports::{UserRepository, UserRoleRepository},
};

use super::entity::{PasswordPolicy, UpdatePasswordPolicy};
use super::error::PasswordPolicyViolation;
use super::ports::PasswordPolicyPolicy;
use super::repository::PasswordPolicyRepository;

#[derive(Debug, Clone)]
pub struct PasswordPolicyService<R, U, C, UR>
where
    R: PasswordPolicyRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
{
    repository: Arc<R>,
    policy: Arc<FerriskeyPolicy<U, C, UR>>,
}

impl<R, U, C, UR> PasswordPolicyService<R, U, C, UR>
where
    R: PasswordPolicyRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
{
    pub fn new(repository: Arc<R>, policy: Arc<FerriskeyPolicy<U, C, UR>>) -> Self {
        Self { repository, policy }
    }

    pub async fn get_policy(
        &self,
        identity: Identity,
        realm: &Realm,
    ) -> Result<PasswordPolicy, CoreError> {
        // Check authorization
        ensure_policy(
            self.policy.can_view_password_policy(&identity, realm).await,
            "view realm password policy",
        )?;

        self.repository
            .find_by_realm_id(realm.id.into())
            .await?
            .map(Ok)
            .unwrap_or_else(|| Ok(PasswordPolicy::default(realm.id.into())))
    }

    pub async fn update_policy(
        &self,
        identity: Identity,
        realm: &Realm,
        update: UpdatePasswordPolicy,
    ) -> Result<PasswordPolicy, CoreError> {
        // Check authorization
        ensure_policy(
            self.policy
                .can_update_password_policy(&identity, realm)
                .await,
            "update realm password policy",
        )?;

        self.repository.upsert(realm.id.into(), update).await
    }

    pub async fn enforce(&self, realm_id: Uuid, password: &str) -> Result<(), CoreError> {
        let policy = self
            .repository
            .find_by_realm_id(realm_id)
            .await?
            .unwrap_or_else(|| PasswordPolicy::default(realm_id));
        policy.validate(password).map_err(|errors| {
            let violations: Vec<PasswordPolicyViolation> = errors.iter().map(Into::into).collect();
            CoreError::PasswordPolicyViolation(
                serde_json::to_string(&violations).unwrap_or_default(),
            )
        })
    }

    pub async fn get_policy_public(&self, realm_id: Uuid) -> Result<PasswordPolicy, CoreError> {
        Ok(self
            .repository
            .find_by_realm_id(realm_id)
            .await?
            .unwrap_or_else(|| PasswordPolicy::default(realm_id)))
    }
}
