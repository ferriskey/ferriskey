use crate::domain::authentication::services::client_secret_matches;
use crate::domain::client::entities::Client;
use crate::domain::common::entities::app_errors::CoreError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientGrant {
    AuthorizationCode,
    RefreshToken,
    Password,
    ClientCredentials,
    DeviceCode,
    Revocation,
    Introspection,
}

pub fn authenticate_client(
    client: &Client,
    presented_secret: Option<&str>,
    grant: ClientGrant,
) -> Result<(), CoreError> {
    if !client.enabled {
        return Err(CoreError::ClientAuthenticationFailed);
    }

    if client.public_client {
        match grant {
            ClientGrant::Introspection => return Err(CoreError::ClientAuthenticationFailed),
            ClientGrant::ClientCredentials => {
                return Err(CoreError::UnauthorizedClient(
                    "A public client cannot use the client_credentials grant.".to_string(),
                ));
            }
            _ => {}
        }
    } else {
        let authenticated = matches!(
            (client.secret_str(), presented_secret),
            (Some(stored), Some(presented)) if client_secret_matches(Some(stored), Some(presented))
        );
        if !authenticated {
            return Err(CoreError::ClientAuthenticationFailed);
        }
    }

    let refusal = match grant {
        ClientGrant::Password if !client.direct_access_grants_enabled => {
            Some("Direct access grants are disabled for this client.")
        }
        ClientGrant::ClientCredentials if !client.service_account_enabled => {
            Some("Service accounts are disabled for this client.")
        }
        ClientGrant::DeviceCode if !client.oauth_device_code_grant_enabled => {
            Some("The device authorization grant is disabled for this client.")
        }
        _ => None,
    };

    match refusal {
        Some(reason) => Err(CoreError::UnauthorizedClient(reason.to_string())),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use uuid::Uuid;

    use super::{ClientGrant, authenticate_client};
    use crate::domain::authentication::entities::AuthProtocol;
    use crate::domain::client::entities::{Client, ClientType, MaintenanceSessionStrategy};
    use crate::domain::common::entities::app_errors::CoreError;
    use crate::domain::realm::entities::RealmId;

    const SECRET: &str = "s3cr3t";

    const ALL_GRANTS: [ClientGrant; 7] = [
        ClientGrant::AuthorizationCode,
        ClientGrant::RefreshToken,
        ClientGrant::Password,
        ClientGrant::ClientCredentials,
        ClientGrant::DeviceCode,
        ClientGrant::Revocation,
        ClientGrant::Introspection,
    ];

    fn client(public: bool) -> Client {
        Client {
            id: Uuid::new_v4(),
            enabled: true,
            client_id: "app".to_string(),
            secret: (!public).then(|| maskass::Masked::new(SECRET.to_string())),
            realm_id: RealmId::from(Uuid::new_v4()),
            protocol: AuthProtocol::OpenIdConnect,
            public_client: public,
            service_account_enabled: true,
            direct_access_grants_enabled: true,
            oauth_device_code_grant_enabled: true,
            token_exchange_enabled: false,
            require_pkce: false,
            client_type: if public {
                ClientType::Public
            } else {
                ClientType::Confidential
            },
            name: "app".to_string(),
            redirect_uris: None,
            access_token_lifetime: None,
            refresh_token_lifetime: None,
            id_token_lifetime: None,
            temporary_token_lifetime: None,
            maintenance_enabled: false,
            maintenance_reason: None,
            maintenance_session_strategy: MaintenanceSessionStrategy::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            backchannel_logout_uri: None,
            backchannel_logout_session_required: true,
            consent_required: false,
        }
    }

    fn is_invalid_client(result: Result<(), CoreError>) -> bool {
        matches!(result, Err(CoreError::ClientAuthenticationFailed))
    }

    fn is_unauthorized_client(result: Result<(), CoreError>) -> bool {
        matches!(result, Err(CoreError::UnauthorizedClient(_)))
    }

    #[test]
    fn a_disabled_client_is_refused_on_every_grant() {
        for public in [false, true] {
            let mut disabled = client(public);
            disabled.enabled = false;
            for grant in ALL_GRANTS {
                assert!(
                    is_invalid_client(authenticate_client(&disabled, Some(SECRET), grant)),
                    "public={public} {grant:?}"
                );
            }
        }
    }

    #[test]
    fn a_confidential_client_with_its_secret_is_accepted_on_every_grant() {
        let confidential = client(false);
        for grant in ALL_GRANTS {
            assert!(
                authenticate_client(&confidential, Some(SECRET), grant).is_ok(),
                "{grant:?}"
            );
        }
    }

    #[test]
    fn a_confidential_client_without_its_secret_is_refused_on_every_grant() {
        let confidential = client(false);
        for presented in [None, Some(""), Some("wrong")] {
            for grant in ALL_GRANTS {
                assert!(
                    is_invalid_client(authenticate_client(&confidential, presented, grant)),
                    "{presented:?} {grant:?}"
                );
            }
        }
    }

    #[test]
    fn a_confidential_client_without_a_registered_secret_is_refused() {
        let mut misconfigured = client(false);
        misconfigured.secret = None;
        for presented in [None, Some(SECRET)] {
            assert!(is_invalid_client(authenticate_client(
                &misconfigured,
                presented,
                ClientGrant::RefreshToken
            )));
        }
    }

    #[test]
    fn a_public_client_needs_no_secret_for_user_grants() {
        let public = client(true);
        for grant in [
            ClientGrant::AuthorizationCode,
            ClientGrant::RefreshToken,
            ClientGrant::Password,
            ClientGrant::DeviceCode,
            ClientGrant::Revocation,
        ] {
            assert!(
                authenticate_client(&public, None, grant).is_ok(),
                "{grant:?}"
            );
        }
    }

    #[test]
    fn a_public_client_cannot_use_client_credentials() {
        assert!(is_unauthorized_client(authenticate_client(
            &client(true),
            None,
            ClientGrant::ClientCredentials
        )));
    }

    #[test]
    fn a_public_client_cannot_introspect() {
        assert!(is_invalid_client(authenticate_client(
            &client(true),
            None,
            ClientGrant::Introspection
        )));
    }

    #[test]
    fn password_needs_direct_access_grants_for_every_client() {
        for public in [false, true] {
            let mut restricted = client(public);
            restricted.direct_access_grants_enabled = false;
            assert!(
                is_unauthorized_client(authenticate_client(
                    &restricted,
                    Some(SECRET),
                    ClientGrant::Password
                )),
                "public={public}"
            );
        }
    }

    #[test]
    fn client_credentials_needs_a_service_account() {
        let mut restricted = client(false);
        restricted.service_account_enabled = false;
        assert!(is_unauthorized_client(authenticate_client(
            &restricted,
            Some(SECRET),
            ClientGrant::ClientCredentials
        )));
    }

    #[test]
    fn the_device_grant_needs_to_be_enabled() {
        for public in [false, true] {
            let mut restricted = client(public);
            restricted.oauth_device_code_grant_enabled = false;
            assert!(
                is_unauthorized_client(authenticate_client(
                    &restricted,
                    Some(SECRET),
                    ClientGrant::DeviceCode
                )),
                "public={public}"
            );
        }
    }

    #[test]
    fn a_wrong_secret_is_refused_before_the_grant_is_checked() {
        let mut restricted = client(false);
        restricted.direct_access_grants_enabled = false;
        assert!(is_invalid_client(authenticate_client(
            &restricted,
            Some("wrong"),
            ClientGrant::Password
        )));
    }
}
