use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::credential::entities::CredentialData;
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
    if !hasher.needs_rehash(algorithm) {
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
