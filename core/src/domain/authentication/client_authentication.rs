use crate::domain::authentication::client_assertion::ports::ClientAssertionVerifier;
use crate::domain::authentication::entities::GrantType;
use crate::domain::authentication::services::client_secret_matches;
use crate::domain::client::entities::Client;
use crate::domain::client::entities::auth_method::TokenEndpointAuthMethod;
use crate::domain::common::entities::app_errors::CoreError;

pub use ferriskey_domain::authentication::client_credential::{
    ClientAssertion, ClientCredential, ClientProof, JWT_BEARER_ASSERTION_TYPE,
};

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

impl ClientGrant {
    pub fn for_token_grant(grant_type: &GrantType) -> Result<Self, CoreError> {
        match grant_type {
            GrantType::Code => Ok(Self::AuthorizationCode),
            GrantType::Password => Ok(Self::Password),
            GrantType::Credentials => Ok(Self::ClientCredentials),
            GrantType::RefreshToken => Ok(Self::RefreshToken),
            GrantType::DeviceCode => Ok(Self::DeviceCode),
            GrantType::TokenExchange => Err(CoreError::InvalidRequest),
        }
    }
}

/// Checks what the client presented and returns how it proved itself.
///
/// A public client proves nothing, whatever it sent. A confidential client
/// must present the credential of its registered method: a secret for
/// `client_secret_basic` and `client_secret_post`, an assertion for the two
/// JWT methods.
pub async fn prove_client<V: ClientAssertionVerifier>(
    verifier: &V,
    client: &Client,
    credential: &ClientCredential,
) -> Result<ClientProof, CoreError> {
    if client.public_client {
        return Ok(ClientProof::None);
    }

    match credential {
        ClientCredential::None => Err(CoreError::ClientAuthenticationFailed),
        ClientCredential::Secret(presented) => {
            let accepts_secret = matches!(
                client.token_endpoint_auth_method,
                TokenEndpointAuthMethod::ClientSecretBasic
                    | TokenEndpointAuthMethod::ClientSecretPost
            );
            let matches = matches!(
                client.secret_str(),
                Some(stored) if client_secret_matches(Some(stored), Some(presented.expose()))
            );
            if accepts_secret && matches {
                Ok(ClientProof::Secret)
            } else {
                Err(CoreError::ClientAuthenticationFailed)
            }
        }
        ClientCredential::Assertion(assertion) => verifier.verify(client, assertion).await,
    }
}

/// Whether a client that proved itself with `proof` may use `grant`.
pub fn authorize_client(
    client: &Client,
    proof: ClientProof,
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
    } else if !proof_fits(client.token_endpoint_auth_method, proof) {
        return Err(CoreError::ClientAuthenticationFailed);
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

pub async fn authenticate_client<V: ClientAssertionVerifier>(
    verifier: &V,
    client: &Client,
    credential: &ClientCredential,
    grant: ClientGrant,
) -> Result<ClientProof, CoreError> {
    if !client.enabled {
        return Err(CoreError::ClientAuthenticationFailed);
    }

    let proof = prove_client(verifier, client, credential).await?;
    authorize_client(client, proof, grant)?;
    Ok(proof)
}

fn proof_fits(method: TokenEndpointAuthMethod, proof: ClientProof) -> bool {
    matches!(
        (method, proof),
        (
            TokenEndpointAuthMethod::ClientSecretBasic | TokenEndpointAuthMethod::ClientSecretPost,
            ClientProof::Secret
        ) | (
            TokenEndpointAuthMethod::ClientSecretJwt,
            ClientProof::SecretJwt
        ) | (
            TokenEndpointAuthMethod::PrivateKeyJwt,
            ClientProof::PrivateKeyJwt
        )
    )
}

#[cfg(test)]
pub(crate) mod tests {
    use chrono::Utc;
    use uuid::Uuid;

    use super::{
        ClientAssertion, ClientCredential, ClientGrant, ClientProof, authenticate_client,
        authorize_client,
    };
    use crate::domain::authentication::client_assertion::ports::ClientAssertionVerifier;
    use crate::domain::authentication::entities::AuthProtocol;
    use crate::domain::client::entities::auth_method::TokenEndpointAuthMethod;
    use crate::domain::client::entities::{
        Client, ClientRegistrationSource, ClientType, MaintenanceSessionStrategy,
    };
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

    pub(crate) fn client(public: bool) -> Client {
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
            registration_source: ClientRegistrationSource::Admin,
            token_endpoint_auth_method: TokenEndpointAuthMethod::default_for(public),
            jwks: None,
            jwks_uri: None,
        }
    }

    struct FixedVerifier(Result<ClientProof, CoreError>);

    impl ClientAssertionVerifier for FixedVerifier {
        async fn verify(
            &self,
            _client: &Client,
            _assertion: &ClientAssertion,
        ) -> Result<ClientProof, CoreError> {
            self.0.clone()
        }
    }

    const NO_ASSERTIONS: FixedVerifier = FixedVerifier(Err(CoreError::ClientAuthenticationFailed));

    fn credential(secret: Option<&str>) -> ClientCredential {
        ClientCredential::from_secret(secret.map(str::to_string))
    }

    fn assertion() -> ClientCredential {
        ClientCredential::Assertion(ClientAssertion::new("a.b.c".to_string(), vec![]))
    }

    async fn authenticate(
        client: &Client,
        secret: Option<&str>,
        grant: ClientGrant,
    ) -> Result<(), CoreError> {
        authenticate_client(&NO_ASSERTIONS, client, &credential(secret), grant)
            .await
            .map(|_| ())
    }

    fn is_invalid_client(result: Result<(), CoreError>) -> bool {
        matches!(result, Err(CoreError::ClientAuthenticationFailed))
    }

    fn is_unauthorized_client(result: Result<(), CoreError>) -> bool {
        matches!(result, Err(CoreError::UnauthorizedClient(_)))
    }

    fn with_method(method: TokenEndpointAuthMethod) -> Client {
        let mut client = client(false);
        client.token_endpoint_auth_method = method;
        client
    }

    #[tokio::test]
    async fn a_disabled_client_is_refused_on_every_grant() {
        for public in [false, true] {
            let mut disabled = client(public);
            disabled.enabled = false;
            for grant in ALL_GRANTS {
                assert!(
                    is_invalid_client(authenticate(&disabled, Some(SECRET), grant).await),
                    "public={public} {grant:?}"
                );
            }
        }
    }

    #[tokio::test]
    async fn a_confidential_client_with_its_secret_is_accepted_on_every_grant() {
        for method in [
            TokenEndpointAuthMethod::ClientSecretBasic,
            TokenEndpointAuthMethod::ClientSecretPost,
        ] {
            let confidential = with_method(method);
            for grant in ALL_GRANTS {
                assert!(
                    authenticate(&confidential, Some(SECRET), grant)
                        .await
                        .is_ok(),
                    "{method} {grant:?}"
                );
            }
        }
    }

    #[tokio::test]
    async fn a_confidential_client_without_its_secret_is_refused_on_every_grant() {
        let confidential = client(false);
        for presented in [None, Some(""), Some("wrong")] {
            for grant in ALL_GRANTS {
                assert!(
                    is_invalid_client(authenticate(&confidential, presented, grant).await),
                    "{presented:?} {grant:?}"
                );
            }
        }
    }

    #[tokio::test]
    async fn a_confidential_client_without_a_registered_secret_is_refused() {
        let mut misconfigured = client(false);
        misconfigured.secret = None;
        for presented in [None, Some(SECRET)] {
            assert!(is_invalid_client(
                authenticate(&misconfigured, presented, ClientGrant::RefreshToken).await
            ));
        }
    }

    #[tokio::test]
    async fn a_jwt_client_cannot_fall_back_to_its_secret() {
        for method in [
            TokenEndpointAuthMethod::ClientSecretJwt,
            TokenEndpointAuthMethod::PrivateKeyJwt,
        ] {
            assert!(
                is_invalid_client(
                    authenticate(
                        &with_method(method),
                        Some(SECRET),
                        ClientGrant::RefreshToken
                    )
                    .await
                ),
                "{method}"
            );
        }
    }

    #[tokio::test]
    async fn an_assertion_is_left_to_the_verifier() {
        let accepting = FixedVerifier(Ok(ClientProof::PrivateKeyJwt));
        let client = with_method(TokenEndpointAuthMethod::PrivateKeyJwt);

        assert!(matches!(
            authenticate_client(
                &accepting,
                &client,
                &assertion(),
                ClientGrant::ClientCredentials
            )
            .await,
            Ok(ClientProof::PrivateKeyJwt)
        ));
        assert!(matches!(
            authenticate_client(
                &NO_ASSERTIONS,
                &client,
                &assertion(),
                ClientGrant::ClientCredentials
            )
            .await,
            Err(CoreError::ClientAuthenticationFailed)
        ));
    }

    #[test]
    fn a_proof_must_match_the_registered_method() {
        let cases = [
            (
                TokenEndpointAuthMethod::ClientSecretBasic,
                ClientProof::Secret,
                true,
            ),
            (
                TokenEndpointAuthMethod::ClientSecretPost,
                ClientProof::Secret,
                true,
            ),
            (
                TokenEndpointAuthMethod::ClientSecretJwt,
                ClientProof::SecretJwt,
                true,
            ),
            (
                TokenEndpointAuthMethod::PrivateKeyJwt,
                ClientProof::PrivateKeyJwt,
                true,
            ),
            (
                TokenEndpointAuthMethod::ClientSecretBasic,
                ClientProof::PrivateKeyJwt,
                false,
            ),
            (
                TokenEndpointAuthMethod::PrivateKeyJwt,
                ClientProof::SecretJwt,
                false,
            ),
            (
                TokenEndpointAuthMethod::PrivateKeyJwt,
                ClientProof::Secret,
                false,
            ),
            (
                TokenEndpointAuthMethod::ClientSecretJwt,
                ClientProof::None,
                false,
            ),
        ];
        for (method, proof, accepted) in cases {
            assert_eq!(
                authorize_client(&with_method(method), proof, ClientGrant::RefreshToken).is_ok(),
                accepted,
                "{method} {proof:?}"
            );
        }
    }

    #[tokio::test]
    async fn a_public_client_needs_no_secret_for_user_grants() {
        let public = client(true);
        for grant in [
            ClientGrant::AuthorizationCode,
            ClientGrant::RefreshToken,
            ClientGrant::Password,
            ClientGrant::DeviceCode,
            ClientGrant::Revocation,
        ] {
            assert!(
                authenticate(&public, None, grant).await.is_ok(),
                "{grant:?}"
            );
        }
    }

    #[tokio::test]
    async fn a_public_client_cannot_use_client_credentials() {
        assert!(is_unauthorized_client(
            authenticate(&client(true), None, ClientGrant::ClientCredentials).await
        ));
    }

    #[tokio::test]
    async fn a_public_client_cannot_introspect() {
        assert!(is_invalid_client(
            authenticate(&client(true), None, ClientGrant::Introspection).await
        ));
    }

    #[tokio::test]
    async fn password_needs_direct_access_grants_for_every_client() {
        for public in [false, true] {
            let mut restricted = client(public);
            restricted.direct_access_grants_enabled = false;
            assert!(
                is_unauthorized_client(
                    authenticate(&restricted, Some(SECRET), ClientGrant::Password).await
                ),
                "public={public}"
            );
        }
    }

    #[tokio::test]
    async fn client_credentials_needs_a_service_account() {
        let mut restricted = client(false);
        restricted.service_account_enabled = false;
        assert!(is_unauthorized_client(
            authenticate(&restricted, Some(SECRET), ClientGrant::ClientCredentials).await
        ));
    }

    #[tokio::test]
    async fn the_device_grant_needs_to_be_enabled() {
        for public in [false, true] {
            let mut restricted = client(public);
            restricted.oauth_device_code_grant_enabled = false;
            assert!(
                is_unauthorized_client(
                    authenticate(&restricted, Some(SECRET), ClientGrant::DeviceCode).await
                ),
                "public={public}"
            );
        }
    }

    #[tokio::test]
    async fn a_wrong_secret_is_refused_before_the_grant_is_checked() {
        let mut restricted = client(false);
        restricted.direct_access_grants_enabled = false;
        assert!(is_invalid_client(
            authenticate(&restricted, Some("wrong"), ClientGrant::Password).await
        ));
    }
}
