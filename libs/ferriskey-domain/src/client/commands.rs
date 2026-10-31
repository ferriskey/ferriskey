use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

use crate::authentication::entities::AuthProtocol;
use crate::client::entities::ClientType;
use crate::client::value_objects::{
    CreateSamlAttributeMapperRequest, CreateTokenExchangePolicyRequest, CreateWebOriginRequest,
    SetClientSamlConfigRequest, UpdateClientRequest,
};
use crate::common::app_errors::CoreError;
use crate::realm::RealmId;

pub struct CreateClientInput {
    pub realm_name: String,
    pub name: String,
    pub client_id: String,
    pub client_type: ClientType,
    pub service_account_enabled: bool,
    pub public_client: bool,
    pub protocol: AuthProtocol,
    pub enabled: bool,
    pub direct_access_grants_enabled: bool,
    pub oauth_device_code_grant_enabled: bool,
    pub token_exchange_enabled: bool,
    pub consent_required: bool,
    pub require_pkce: bool,
}

pub struct CreateRedirectUriInput {
    pub client_id: Uuid,
    pub realm_name: String,
    pub payload: crate::client::value_objects::CreateRedirectUriRequest,
}

pub struct CreatePostLogoutRedirectUriInput {
    pub client_id: Uuid,
    pub realm_name: String,
    pub payload: crate::client::value_objects::CreateRedirectUriRequest,
}

pub struct CreateRoleInput {
    pub realm_name: String,
    pub client_id: Uuid,
    pub description: Option<String>,
    pub name: String,
    pub permissions: Vec<String>,
}

pub struct DeleteClientInput {
    pub realm_name: String,
    pub client_id: Uuid,
}

pub struct DeleteRedirectUriInput {
    pub realm_name: String,
    pub client_id: Uuid,
    pub uri_id: Uuid,
}

pub struct DeletePostLogoutRedirectUriInput {
    pub realm_name: String,
    pub client_id: Uuid,
    pub uri_id: Uuid,
}

pub struct GetClientInput {
    pub client_id: Uuid,
    pub realm_name: String,
}

/// The longest a previous secret may outlive its rotation.
pub const MAX_SECRET_GRACE_PERIOD_SECS: i64 = 30 * 24 * 60 * 60;

pub struct RotateClientSecretInput {
    pub client_id: Uuid,
    pub realm_name: String,
    /// How long the secret being replaced keeps working. `None` or `0` cuts it
    /// off immediately.
    pub grace_period_secs: Option<i64>,
}

impl RotateClientSecretInput {
    pub fn grace_period(&self) -> Result<Option<Duration>, CoreError> {
        match self.grace_period_secs {
            None | Some(0) => Ok(None),
            Some(secs) if (1..=MAX_SECRET_GRACE_PERIOD_SECS).contains(&secs) => {
                Ok(Some(Duration::seconds(secs)))
            }
            Some(secs) => Err(CoreError::InvalidSecretGracePeriod(format!(
                "{secs} seconds is outside 0..={MAX_SECRET_GRACE_PERIOD_SECS}"
            ))),
        }
    }
}

pub struct RotatedClientSecret {
    pub client_secret: String,
    pub previous_secret_expires_at: Option<DateTime<Utc>>,
}

pub struct RevealedPreviousSecret {
    pub client_secret: String,
    pub expires_at: DateTime<Utc>,
}

pub struct RevealedClientSecret {
    pub client_secret: Option<String>,
    pub previous: Option<RevealedPreviousSecret>,
}

pub struct ExpiredClientSecret {
    pub client_id: Uuid,
    pub realm_id: RealmId,
    pub expired_at: DateTime<Utc>,
}

pub struct GetClientRolesInput {
    pub client_id: Uuid,
    pub realm_name: String,
}

pub struct GetRedirectUrisInput {
    pub realm_name: String,
    pub client_id: Uuid,
}

pub struct GetPostLogoutRedirectUrisInput {
    pub realm_name: String,
    pub client_id: Uuid,
}

pub struct UpdateClientInput {
    pub realm_name: String,
    pub client_id: Uuid,
    pub payload: UpdateClientRequest,
}

pub struct UpdateRedirectUriInput {
    pub realm_name: String,
    pub client_id: Uuid,
    pub redirect_uri_id: Uuid,
    pub enabled: bool,
}

pub struct UpdatePostLogoutRedirectUriInput {
    pub realm_name: String,
    pub client_id: Uuid,
    pub redirect_uri_id: Uuid,
    pub enabled: bool,
}

pub struct CreateWebOriginInput {
    pub client_id: Uuid,
    pub realm_name: String,
    pub payload: CreateWebOriginRequest,
}

pub struct GetWebOriginsInput {
    pub client_id: Uuid,
    pub realm_name: String,
}

pub struct DeleteWebOriginInput {
    pub realm_name: String,
    pub client_id: Uuid,
    pub web_origin_id: Uuid,
}

pub struct CreateTokenExchangePolicyInput {
    pub realm_name: String,
    pub client_id: Uuid,
    pub payload: CreateTokenExchangePolicyRequest,
}

pub struct GetTokenExchangePoliciesInput {
    pub realm_name: String,
    pub client_id: Uuid,
}

pub struct DeleteTokenExchangePolicyInput {
    pub realm_name: String,
    pub client_id: Uuid,
    pub policy_id: Uuid,
}

pub struct GetClientSamlConfigInput {
    pub realm_name: String,
    pub client_id: Uuid,
}

pub struct SetClientSamlConfigInput {
    pub realm_name: String,
    pub client_id: Uuid,
    pub payload: SetClientSamlConfigRequest,
}

pub struct CreateSamlAttributeMapperInput {
    pub realm_name: String,
    pub client_id: Uuid,
    pub payload: CreateSamlAttributeMapperRequest,
}

pub struct GetSamlAttributeMappersInput {
    pub realm_name: String,
    pub client_id: Uuid,
}

pub struct DeleteSamlAttributeMapperInput {
    pub realm_name: String,
    pub client_id: Uuid,
    pub mapper_id: Uuid,
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;

    fn rotation(grace_period_secs: Option<i64>) -> RotateClientSecretInput {
        RotateClientSecretInput {
            client_id: Uuid::new_v4(),
            realm_name: "acme".to_string(),
            grace_period_secs,
        }
    }

    #[test]
    fn no_grace_period_cuts_the_old_secret_off() {
        assert!(matches!(rotation(None).grace_period(), Ok(None)));
        assert!(matches!(rotation(Some(0)).grace_period(), Ok(None)));
    }

    #[test]
    fn a_grace_period_up_to_the_maximum_is_kept() {
        assert!(matches!(
            rotation(Some(86_400)).grace_period(),
            Ok(Some(grace)) if grace == Duration::days(1)
        ));
        assert!(matches!(
            rotation(Some(MAX_SECRET_GRACE_PERIOD_SECS)).grace_period(),
            Ok(Some(grace)) if grace == Duration::days(30)
        ));
    }

    #[test]
    fn a_negative_or_oversized_grace_period_is_refused() {
        for secs in [-1, MAX_SECRET_GRACE_PERIOD_SECS + 1, i64::MAX, i64::MIN] {
            assert!(
                matches!(
                    rotation(Some(secs)).grace_period(),
                    Err(CoreError::InvalidSecretGracePeriod(_))
                ),
                "{secs}"
            );
        }
    }
}
