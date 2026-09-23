use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::credential::entities::CredentialData;
use ferriskey_domain::credential::ports::CredentialRepository;
use tracing::{Instrument, info_span};
use uuid::Uuid;

use crate::crypto::ports::HasherRepository;

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

    let salt = credential.salt.ok_or(CoreError::InternalServerError)?;

    let CredentialData::Hash {
        hash_iterations,
        algorithm,
    } = credential.credential_data
    else {
        return Err(CoreError::InternalServerError);
    };

    hasher
        .verify_password(
            password,
            &credential.secret_data,
            hash_iterations,
            &algorithm,
            &salt,
        )
        .instrument(info_span!(
            "auth.verify_password.hasher_verify",
            hash_algorithm = %algorithm,
            hash_iterations
        ))
        .await
        .map_err(|_| CoreError::InternalServerError)
}

pub async fn has_local_password<C>(credentials: &C, user_id: Uuid) -> Result<bool, CoreError>
where
    C: CredentialRepository,
{
    let credential = match credentials.get_password_credential(user_id).await {
        Ok(credential) => credential,
        Err(_) => return Ok(false),
    };

    Ok(credential.salt.is_some()
        && matches!(credential.credential_data, CredentialData::Hash { .. }))
}
