use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::credential::entities::{Credential, CredentialData};
use ferriskey_domain::credential::ports::CredentialRepository;
use tracing::{Instrument, info_span, warn};
use uuid::Uuid;

use crate::SecurityError;
use crate::crypto::ports::HasherRepository;

pub async fn verify_password_hash<H: HasherRepository>(
    hasher: &H,
    secret_data: &str,
    salt: Option<&str>,
    hash_iterations: u32,
    algorithm: &str,
    password: &str,
) -> Result<bool, SecurityError> {
    hasher
        .verify_password(
            password,
            secret_data,
            hash_iterations,
            algorithm,
            salt.unwrap_or_default(),
        )
        .await
}

pub async fn rehash_password_if_needed<H: HasherRepository, CR: CredentialRepository>(
    hasher: &H,
    credential_repository: &CR,
    user_id: Uuid,
    verified_secret_data: &str,
    password: &str,
    algorithm: &str,
    temporary: bool,
) {
    if !hasher.needs_rehash(algorithm, verified_secret_data) {
        return;
    }

    let hash_result = match hasher.hash_password(password).await {
        Ok(hash_result) => hash_result,
        Err(e) => {
            warn!(%user_id, from = %algorithm, "password rehash failed: {e}");
            return;
        }
    };

    if let Err(e) = credential_repository
        .update_password_credential(user_id, verified_secret_data, hash_result, temporary)
        .await
    {
        warn!(%user_id, from = %algorithm, "password rehash not persisted: {e:?}");
    }
}

pub async fn verify_user_password<C, H>(
    credentials: &C,
    hasher: &H,
    user_id: Uuid,
    password: &str,
) -> Result<bool, CoreError>
where
    C: CredentialRepository,
    H: HasherRepository,
{
    let credential = credentials
        .get_password_credential(user_id)
        .instrument(info_span!("auth.verify_password.credential_fetch"))
        .await
        .map_err(|_| CoreError::InternalServerError)?;

    let temporary = credential.temporary;

    let CredentialData::Hash {
        hash_iterations,
        algorithm,
    } = credential.credential_data
    else {
        return Err(CoreError::InternalServerError);
    };

    let is_valid = verify_password_hash(
        hasher,
        &credential.secret_data,
        credential.salt.as_deref(),
        hash_iterations,
        &algorithm,
        password,
    )
    .instrument(info_span!(
        "auth.verify_password.hasher_verify",
        hash_algorithm = %algorithm,
        hash_iterations
    ))
    .await
    .map_err(|_| CoreError::InternalServerError)?;

    if is_valid {
        rehash_password_if_needed(
            hasher,
            credentials,
            user_id,
            &credential.secret_data,
            password,
            &algorithm,
            temporary,
        )
        .await;
    }

    Ok(is_valid)
}

pub async fn verify_password_credential<H, C>(
    hasher: &H,
    credentials: &C,
    credential: Option<&Credential>,
    password: &str,
) -> bool
where
    H: HasherRepository,
    C: CredentialRepository,
{
    let Some(credential) = credential else {
        return false;
    };

    let CredentialData::Hash {
        hash_iterations,
        algorithm,
    } = &credential.credential_data
    else {
        warn!(user_id = %credential.user_id, "the password credential carries no hash");
        return false;
    };

    let is_valid = match verify_password_hash(
        hasher,
        &credential.secret_data,
        credential.salt.as_deref(),
        *hash_iterations,
        algorithm,
        password,
    )
    .instrument(info_span!(
        "auth.verify_password.hasher_verify",
        hash_algorithm = %algorithm,
        hash_iterations
    ))
    .await
    {
        Ok(is_valid) => is_valid,
        Err(e) => {
            warn!(user_id = %credential.user_id, "password verification failed: {e}");
            return false;
        }
    };

    if is_valid {
        rehash_password_if_needed(
            hasher,
            credentials,
            credential.user_id,
            &credential.secret_data,
            password,
            algorithm,
            credential.temporary,
        )
        .await;
    }

    is_valid
}

pub async fn has_local_password<C>(credentials: &C, user_id: Uuid) -> Result<bool, CoreError>
where
    C: CredentialRepository,
{
    let credential = match credentials.get_password_credential(user_id).await {
        Ok(credential) => credential,
        Err(_) => return Ok(false),
    };

    Ok(matches!(
        credential.credential_data,
        CredentialData::Hash { .. }
    ))
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use ferriskey_domain::credential::entities::{Credential, CredentialData, CredentialType};
    use ferriskey_domain::credential::ports::MockCredentialRepository;
    use uuid::Uuid;

    use super::verify_password_credential;
    use crate::SecurityError;
    use crate::crypto::ports::MockHasherRepository;

    fn credential(data: CredentialData) -> Credential {
        Credential {
            id: Uuid::new_v4(),
            salt: None,
            credential_type: CredentialType::Password,
            user_id: Uuid::new_v4(),
            user_label: None,
            secret_data: "$argon2id$stored".to_string(),
            credential_data: data,
            temporary: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            webauthn_credential_id: None,
        }
    }

    fn hashed() -> Credential {
        credential(CredentialData::new_hash(5, "argon2id".to_string()))
    }

    fn hasher_answering(answer: Option<bool>) -> MockHasherRepository {
        let mut hasher = MockHasherRepository::new();
        hasher
            .expect_verify_password()
            .times(1)
            .returning(move |_, _, _, _, _| {
                Box::pin(async move {
                    answer
                        .ok_or_else(|| SecurityError::HashingError("not a PHC string".to_string()))
                })
            });
        hasher.expect_needs_rehash().returning(|_, _| false);
        hasher
    }

    #[tokio::test]
    async fn a_matching_password_is_valid() {
        let hasher = hasher_answering(Some(true));

        assert!(
            verify_password_credential(
                &hasher,
                &MockCredentialRepository::new(),
                Some(&hashed()),
                "secret",
            )
            .await
        );
    }

    #[tokio::test]
    async fn a_wrong_password_is_invalid() {
        let hasher = hasher_answering(Some(false));

        assert!(
            !verify_password_credential(
                &hasher,
                &MockCredentialRepository::new(),
                Some(&hashed()),
                "secret",
            )
            .await
        );
    }

    #[tokio::test]
    async fn an_account_without_a_password_credential_never_reaches_the_hasher() {
        let hasher = MockHasherRepository::new();

        assert!(
            !verify_password_credential(&hasher, &MockCredentialRepository::new(), None, "secret")
                .await
        );
    }

    #[tokio::test]
    async fn a_credential_without_a_hash_is_invalid_without_reaching_the_hasher() {
        let hasher = MockHasherRepository::new();
        let federated = credential(CredentialData::Federated {
            provider_id: Uuid::new_v4().to_string(),
            provider_type: "ldap".to_string(),
        });

        assert!(
            !verify_password_credential(
                &hasher,
                &MockCredentialRepository::new(),
                Some(&federated),
                "secret",
            )
            .await
        );
    }

    #[tokio::test]
    async fn a_hasher_failure_is_treated_as_an_invalid_password() {
        let hasher = hasher_answering(None);

        assert!(
            !verify_password_credential(
                &hasher,
                &MockCredentialRepository::new(),
                Some(&hashed()),
                "secret",
            )
            .await
        );
    }
}
