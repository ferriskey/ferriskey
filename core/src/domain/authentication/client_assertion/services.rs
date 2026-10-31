use std::sync::Arc;

use chrono::{DateTime, Utc};
use jsonwebtoken::jwk::{Jwk, JwkSet, PublicKeyUse};
use jsonwebtoken::{Algorithm, DecodingKey, Header, Validation};
use tracing::warn;

use crate::domain::authentication::client_assertion::entities::{
    ASSERTION_LEEWAY_SECONDS, AssertionClaims, CLIENT_SECRET_JWT_ALGORITHMS, ClientAssertionError,
    MAX_ASSERTION_LIFETIME_SECONDS, MAX_JTI_CHARS, PRIVATE_KEY_JWT_ALGORITHMS,
};
use crate::domain::authentication::client_assertion::ports::{
    ClientAssertionReplayStore, ClientAssertionVerifier, ClientJwksFetcher,
};
use crate::domain::authentication::client_authentication::{ClientAssertion, ClientProof};
use crate::domain::client::entities::Client;
use crate::domain::client::entities::auth_method::{ClientKeySource, TokenEndpointAuthMethod};
use crate::domain::common::entities::app_errors::CoreError;

#[derive(Debug)]
pub struct ClientAssertionVerifierImpl<J, S>
where
    J: ClientJwksFetcher,
    S: ClientAssertionReplayStore,
{
    jwks_fetcher: Arc<J>,
    replay_store: Arc<S>,
}

impl<J, S> Clone for ClientAssertionVerifierImpl<J, S>
where
    J: ClientJwksFetcher,
    S: ClientAssertionReplayStore,
{
    fn clone(&self) -> Self {
        Self {
            jwks_fetcher: Arc::clone(&self.jwks_fetcher),
            replay_store: Arc::clone(&self.replay_store),
        }
    }
}

impl<J, S> ClientAssertionVerifierImpl<J, S>
where
    J: ClientJwksFetcher,
    S: ClientAssertionReplayStore,
{
    pub fn new(jwks_fetcher: Arc<J>, replay_store: Arc<S>) -> Self {
        Self {
            jwks_fetcher,
            replay_store,
        }
    }

    async fn check(
        &self,
        client: &Client,
        assertion: &ClientAssertion,
    ) -> Result<ClientProof, ClientAssertionError> {
        let jwt = assertion.jwt.expose();
        let header =
            jsonwebtoken::decode_header(jwt).map_err(|_| ClientAssertionError::Malformed)?;

        let (proof, key) = match client.token_endpoint_auth_method {
            TokenEndpointAuthMethod::PrivateKeyJwt => {
                if !PRIVATE_KEY_JWT_ALGORITHMS.contains(&header.alg) {
                    return Err(ClientAssertionError::UnsupportedAlgorithm);
                }
                (
                    ClientProof::PrivateKeyJwt,
                    self.client_key(client, &header).await?,
                )
            }
            TokenEndpointAuthMethod::ClientSecretJwt => {
                if !CLIENT_SECRET_JWT_ALGORITHMS.contains(&header.alg) {
                    return Err(ClientAssertionError::UnsupportedAlgorithm);
                }
                let secret = client.secret_str().ok_or(ClientAssertionError::NoKeys)?;
                (
                    ClientProof::SecretJwt,
                    DecodingKey::from_secret(secret.as_bytes()),
                )
            }
            _ => return Err(ClientAssertionError::MethodMismatch),
        };

        let claims = verified_claims(jwt, &key, header.alg, &client.client_id, assertion)?;
        let expires_at = bounded_expiry(claims.exp, Utc::now())?;

        let jti = claims
            .jti
            .filter(|jti| !jti.is_empty() && jti.chars().count() <= MAX_JTI_CHARS)
            .ok_or(ClientAssertionError::MissingJti)?;

        let first_use = self
            .replay_store
            .record(client.id, &jti, expires_at)
            .await
            .map_err(|_| ClientAssertionError::Store)?;

        if !first_use {
            return Err(ClientAssertionError::Replayed);
        }

        Ok(proof)
    }

    async fn client_key(
        &self,
        client: &Client,
        header: &Header,
    ) -> Result<DecodingKey, ClientAssertionError> {
        match client.key_source().ok_or(ClientAssertionError::NoKeys)? {
            ClientKeySource::Inline(jwks) => {
                let set: JwkSet = serde_json::from_value(jwks.0.clone())
                    .map_err(|_| ClientAssertionError::NoKeys)?;
                select_key(&set, header)
            }
            ClientKeySource::Uri(uri) => {
                let set = self.jwks_fetcher.fetch(uri, false).await?;
                match select_key(&set, header) {
                    Err(ClientAssertionError::UnknownKey) => {
                        let set = self.jwks_fetcher.fetch(uri, true).await?;
                        select_key(&set, header)
                    }
                    selected => selected,
                }
            }
        }
    }
}

impl<J, S> ClientAssertionVerifier for ClientAssertionVerifierImpl<J, S>
where
    J: ClientJwksFetcher,
    S: ClientAssertionReplayStore,
{
    async fn verify(
        &self,
        client: &Client,
        assertion: &ClientAssertion,
    ) -> Result<ClientProof, CoreError> {
        self.check(client, assertion).await.map_err(|reason| {
            warn!(client_id = %client.client_id, %reason, "client assertion refused");
            CoreError::from(reason)
        })
    }
}

/// The key of `set` that signed the assertion: the one named by `kid`, or,
/// without a `kid`, the only signing key of the algorithm's family.
fn select_key(set: &JwkSet, header: &Header) -> Result<DecodingKey, ClientAssertionError> {
    let signs = |jwk: &&Jwk| {
        !matches!(
            jwk.common.public_key_use,
            Some(PublicKeyUse::Encryption | PublicKeyUse::Other(_))
        )
    };

    let jwk = match header.kid.as_deref() {
        Some(kid) => set.find(kid).filter(signs),
        None => {
            let mut candidates = set.keys.iter().filter(signs).filter(|jwk| {
                DecodingKey::from_jwk(jwk).is_ok_and(|key| key.family() == header.alg.family())
            });
            match (candidates.next(), candidates.next()) {
                (Some(only), None) => Some(only),
                _ => None,
            }
        }
    }
    .ok_or(ClientAssertionError::UnknownKey)?;

    if let Some(pinned) = jwk.common.key_algorithm
        && pinned.to_string() != format!("{:?}", header.alg)
    {
        return Err(ClientAssertionError::UnsupportedAlgorithm);
    }

    let key = DecodingKey::from_jwk(jwk).map_err(|_| ClientAssertionError::NoKeys)?;
    if key.family() != header.alg.family() {
        return Err(ClientAssertionError::UnsupportedAlgorithm);
    }

    Ok(key)
}

/// Signature, `iss` = `sub` = the client id, `aud` naming this server, `exp`
/// and `nbf` (RFC 7523 §3).
fn verified_claims(
    jwt: &str,
    key: &DecodingKey,
    alg: Algorithm,
    client_id: &str,
    assertion: &ClientAssertion,
) -> Result<AssertionClaims, ClientAssertionError> {
    let mut validation = Validation::new(alg);
    validation.set_required_spec_claims(&["exp", "iss", "sub", "aud"]);
    validation.set_issuer(&[client_id]);
    validation.sub = Some(client_id.to_string());
    validation.set_audience(&assertion.audiences);
    validation.validate_nbf = true;
    validation.leeway = ASSERTION_LEEWAY_SECONDS;

    jsonwebtoken::decode::<AssertionClaims>(jwt, key, &validation)
        .map(|data| data.claims)
        .map_err(|error| ClientAssertionError::Rejected(error.to_string()))
}

fn bounded_expiry(exp: i64, now: DateTime<Utc>) -> Result<DateTime<Utc>, ClientAssertionError> {
    if exp > now.timestamp() + MAX_ASSERTION_LIFETIME_SECONDS + ASSERTION_LEEWAY_SECONDS as i64 {
        return Err(ClientAssertionError::TooLongLived);
    }

    DateTime::from_timestamp(exp, 0).ok_or(ClientAssertionError::Rejected(
        "exp is out of range".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use chrono::Utc;
    use ferriskey_security::jwt::entities::JwtKeyPair;
    use jsonwebtoken::jwk::JwkSet;
    use jsonwebtoken::{Algorithm, EncodingKey, Header};
    use serde_json::{Value, json};
    use uuid::Uuid;

    use super::*;
    use crate::domain::authentication::client_assertion::ports::{
        MockClientAssertionReplayStore, MockClientJwksFetcher,
    };
    use crate::domain::authentication::client_authentication::tests::client;
    use crate::domain::client::entities::auth_method::ClientJwks;

    type Mutation = fn(&mut Value, i64);

    const AUDIENCE: &str = "https://iam.example/realms/acme";
    const JWKS_URI: &str = "https://app.example/jwks.json";

    struct Signer {
        kid: String,
        encoding: EncodingKey,
        jwk: Value,
    }

    fn signer() -> Signer {
        let (private, public) = JwtKeyPair::generate().expect("generate a key");
        let certificate =
            JwtKeyPair::self_signed_certificate(&private, "test").expect("certificate");
        let pair = JwtKeyPair::from_pem(
            &private,
            &public,
            &certificate,
            Uuid::new_v4(),
            Uuid::new_v4(),
        )
        .expect("key pair");
        let jwk = serde_json::to_value(pair.to_jwk_key().expect("jwk")).expect("json");
        Signer {
            kid: jwk["kid"].as_str().expect("kid").to_string(),
            encoding: EncodingKey::from_rsa_pem(private.as_bytes()).expect("encoding key"),
            jwk,
        }
    }

    fn key_set(signers: &[&Signer]) -> Value {
        json!({ "keys": signers.iter().map(|s| s.jwk.clone()).collect::<Vec<_>>() })
    }

    fn claims(client_id: &str) -> Value {
        let now = Utc::now().timestamp();
        json!({
            "iss": client_id,
            "sub": client_id,
            "aud": AUDIENCE,
            "exp": now + 60,
            "iat": now,
            "jti": Uuid::new_v4().to_string(),
        })
    }

    fn sign(signer: &Signer, claims: &Value) -> ClientAssertion {
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(signer.kid.clone());
        let jwt = jsonwebtoken::encode(&header, claims, &signer.encoding).expect("sign");
        ClientAssertion::new(jwt, vec![AUDIENCE.to_string()])
    }

    fn private_key_client(jwks: Option<Value>, jwks_uri: Option<&str>) -> Client {
        let mut client = client(false);
        client.token_endpoint_auth_method = TokenEndpointAuthMethod::PrivateKeyJwt;
        client.jwks = jwks.map(ClientJwks);
        client.jwks_uri = jwks_uri.map(str::to_string);
        client
    }

    fn accepting_store() -> MockClientAssertionReplayStore {
        let mut store = MockClientAssertionReplayStore::new();
        store
            .expect_record()
            .returning(|_, _, _| Box::pin(async { Ok(true) }));
        store
    }

    fn remembering_store() -> MockClientAssertionReplayStore {
        let seen = Mutex::new(Vec::<String>::new());
        let mut store = MockClientAssertionReplayStore::new();
        store.expect_record().returning(move |_, jti, _| {
            let mut seen = seen.lock().expect("lock");
            let fresh = !seen.iter().any(|known| known == jti);
            seen.push(jti.to_string());
            Box::pin(async move { Ok(fresh) })
        });
        store
    }

    fn no_fetch() -> MockClientJwksFetcher {
        let mut fetcher = MockClientJwksFetcher::new();
        fetcher.expect_fetch().never();
        fetcher
    }

    fn verifier(
        fetcher: MockClientJwksFetcher,
        store: MockClientAssertionReplayStore,
    ) -> ClientAssertionVerifierImpl<MockClientJwksFetcher, MockClientAssertionReplayStore> {
        ClientAssertionVerifierImpl::new(Arc::new(fetcher), Arc::new(store))
    }

    async fn check(
        verifier: &ClientAssertionVerifierImpl<
            MockClientJwksFetcher,
            MockClientAssertionReplayStore,
        >,
        client: &Client,
        assertion: &ClientAssertion,
    ) -> Result<ClientProof, ClientAssertionError> {
        verifier.check(client, assertion).await
    }

    #[tokio::test]
    async fn a_well_formed_private_key_jwt_assertion_is_accepted() {
        let signer = signer();
        let client = private_key_client(Some(key_set(&[&signer])), None);
        let verifier = verifier(no_fetch(), accepting_store());

        let proof = check(&verifier, &client, &sign(&signer, &claims("app"))).await;

        assert_eq!(proof, Ok(ClientProof::PrivateKeyJwt));
    }

    #[tokio::test]
    async fn a_replayed_jti_is_refused() {
        let signer = signer();
        let client = private_key_client(Some(key_set(&[&signer])), None);
        let verifier = verifier(no_fetch(), remembering_store());
        let assertion = sign(&signer, &claims("app"));

        assert!(check(&verifier, &client, &assertion).await.is_ok());
        assert_eq!(
            check(&verifier, &client, &assertion).await,
            Err(ClientAssertionError::Replayed)
        );
    }

    #[tokio::test]
    async fn an_assertion_signed_by_another_key_is_refused() {
        let registered = signer();
        let attacker = signer();
        let client = private_key_client(Some(key_set(&[&registered])), None);
        let verifier = verifier(no_fetch(), accepting_store());

        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(registered.kid.clone());
        let jwt = jsonwebtoken::encode(&header, &claims("app"), &attacker.encoding).expect("sign");
        let assertion = ClientAssertion::new(jwt, vec![AUDIENCE.to_string()]);

        assert!(matches!(
            check(&verifier, &client, &assertion).await,
            Err(ClientAssertionError::Rejected(_))
        ));
    }

    #[tokio::test]
    async fn claims_that_do_not_name_the_client_and_this_server_are_refused() {
        let signer = signer();
        let client = private_key_client(Some(key_set(&[&signer])), None);
        let now = Utc::now().timestamp();
        let cases: [(&str, Mutation); 7] = [
            ("iss differs", |c, _| c["iss"] = json!("other")),
            ("sub differs", |c, _| c["sub"] = json!("other")),
            ("aud elsewhere", |c, _| {
                c["aud"] = json!("https://evil.example")
            }),
            ("expired", |c, now| c["exp"] = json!(now - 120)),
            ("not yet valid", |c, now| c["nbf"] = json!(now + 600)),
            ("no aud", |c, _| {
                c.as_object_mut().expect("object").remove("aud");
            }),
            ("no exp", |c, _| {
                c.as_object_mut().expect("object").remove("exp");
            }),
        ];

        for (case, mutate) in cases {
            let verifier = verifier(no_fetch(), accepting_store());
            let mut claims = claims("app");
            mutate(&mut claims, now);
            let outcome = check(&verifier, &client, &sign(&signer, &claims)).await;
            assert!(
                matches!(outcome, Err(ClientAssertionError::Rejected(_))),
                "{case}: {outcome:?}"
            );
        }
    }

    #[tokio::test]
    async fn an_audience_list_naming_this_server_is_accepted() {
        let signer = signer();
        let client = private_key_client(Some(key_set(&[&signer])), None);
        let verifier = verifier(no_fetch(), accepting_store());
        let mut claims = claims("app");
        claims["aud"] = json!(["https://other.example", AUDIENCE]);

        assert!(
            check(&verifier, &client, &sign(&signer, &claims))
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn a_missing_jti_is_refused() {
        let signer = signer();
        let client = private_key_client(Some(key_set(&[&signer])), None);
        let verifier = verifier(no_fetch(), accepting_store());
        let mut claims = claims("app");
        claims.as_object_mut().expect("object").remove("jti");

        assert_eq!(
            check(&verifier, &client, &sign(&signer, &claims)).await,
            Err(ClientAssertionError::MissingJti)
        );
    }

    #[tokio::test]
    async fn an_assertion_living_longer_than_an_hour_is_refused() {
        let signer = signer();
        let client = private_key_client(Some(key_set(&[&signer])), None);
        let verifier = verifier(no_fetch(), accepting_store());
        let mut claims = claims("app");
        claims["exp"] = json!(Utc::now().timestamp() + 2 * 60 * 60);

        assert_eq!(
            check(&verifier, &client, &sign(&signer, &claims)).await,
            Err(ClientAssertionError::TooLongLived)
        );
    }

    #[tokio::test]
    async fn an_unsigned_or_symmetric_assertion_is_refused_for_private_key_jwt() {
        let signer = signer();
        let client = private_key_client(Some(key_set(&[&signer])), None);
        let verifier = verifier(no_fetch(), accepting_store());

        let hs = jsonwebtoken::encode(
            &Header::new(Algorithm::HS256),
            &claims("app"),
            &EncodingKey::from_secret(b"anything"),
        )
        .expect("sign");
        let none = {
            let mut parts = hs.split('.');
            let _ = parts.next();
            let payload = parts.next().expect("payload");
            format!("eyJhbGciOiJub25lIn0.{payload}.")
        };

        for jwt in [hs, none] {
            let outcome = check(
                &verifier,
                &client,
                &ClientAssertion::new(jwt, vec![AUDIENCE.to_string()]),
            )
            .await;
            assert!(
                matches!(
                    outcome,
                    Err(ClientAssertionError::UnsupportedAlgorithm
                        | ClientAssertionError::Malformed)
                ),
                "{outcome:?}"
            );
        }
    }

    #[tokio::test]
    async fn an_assertion_from_a_secret_client_is_refused() {
        let signer = signer();
        let mut client = private_key_client(Some(key_set(&[&signer])), None);
        client.token_endpoint_auth_method = TokenEndpointAuthMethod::ClientSecretBasic;
        let verifier = verifier(no_fetch(), accepting_store());

        assert_eq!(
            check(&verifier, &client, &sign(&signer, &claims("app"))).await,
            Err(ClientAssertionError::MethodMismatch)
        );
    }

    #[tokio::test]
    async fn client_secret_jwt_is_signed_with_the_client_secret() {
        let mut client = client(false);
        client.token_endpoint_auth_method = TokenEndpointAuthMethod::ClientSecretJwt;
        let secret = client.secret_str().expect("secret").to_string();
        let verifier = verifier(no_fetch(), accepting_store());

        let sign_with = |key: &[u8]| {
            let jwt = jsonwebtoken::encode(
                &Header::new(Algorithm::HS256),
                &claims("app"),
                &EncodingKey::from_secret(key),
            )
            .expect("sign");
            ClientAssertion::new(jwt, vec![AUDIENCE.to_string()])
        };

        assert_eq!(
            check(&verifier, &client, &sign_with(secret.as_bytes())).await,
            Ok(ClientProof::SecretJwt)
        );
        assert!(matches!(
            check(&verifier, &client, &sign_with(b"not-the-secret")).await,
            Err(ClientAssertionError::Rejected(_))
        ));
    }

    #[tokio::test]
    async fn a_jwks_uri_is_refetched_once_for_an_unknown_kid() {
        let old = signer();
        let rotated = signer();
        let client = private_key_client(None, Some(JWKS_URI));

        let mut fetcher = MockClientJwksFetcher::new();
        let stale: JwkSet = serde_json::from_value(key_set(&[&old])).expect("set");
        let fresh: JwkSet = serde_json::from_value(key_set(&[&old, &rotated])).expect("set");
        fetcher
            .expect_fetch()
            .withf(|uri, refresh| uri == JWKS_URI && !refresh)
            .times(1)
            .returning(move |_, _| {
                let stale = stale.clone();
                Box::pin(async move { Ok(stale) })
            });
        fetcher
            .expect_fetch()
            .withf(|uri, refresh| uri == JWKS_URI && *refresh)
            .times(1)
            .returning(move |_, _| {
                let fresh = fresh.clone();
                Box::pin(async move { Ok(fresh) })
            });
        let verifier = verifier(fetcher, accepting_store());

        assert_eq!(
            check(&verifier, &client, &sign(&rotated, &claims("app"))).await,
            Ok(ClientProof::PrivateKeyJwt)
        );
    }

    #[tokio::test]
    async fn a_known_kid_is_served_from_the_first_fetch() {
        let signer = signer();
        let client = private_key_client(None, Some(JWKS_URI));
        let set: JwkSet = serde_json::from_value(key_set(&[&signer])).expect("set");
        let mut fetcher = MockClientJwksFetcher::new();
        fetcher
            .expect_fetch()
            .withf(|_, refresh| !refresh)
            .times(1)
            .returning(move |_, _| {
                let set = set.clone();
                Box::pin(async move { Ok(set) })
            });
        let verifier = verifier(fetcher, accepting_store());

        assert!(
            check(&verifier, &client, &sign(&signer, &claims("app")))
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn without_a_kid_the_only_key_of_the_family_is_used() {
        let signer = signer();
        let client = private_key_client(Some(key_set(&[&signer])), None);
        let verifier = verifier(no_fetch(), accepting_store());
        let jwt = jsonwebtoken::encode(
            &Header::new(Algorithm::RS256),
            &claims("app"),
            &signer.encoding,
        )
        .expect("sign");

        assert!(
            check(
                &verifier,
                &client,
                &ClientAssertion::new(jwt, vec![AUDIENCE.to_string()])
            )
            .await
            .is_ok()
        );
    }

    #[tokio::test]
    async fn verification_failures_are_invalid_client() {
        let signer = signer();
        let client = private_key_client(Some(key_set(&[&signer])), None);
        let verifier = verifier(no_fetch(), accepting_store());
        let mut claims = claims("app");
        claims["iss"] = json!("other");

        assert!(matches!(
            verifier.verify(&client, &sign(&signer, &claims)).await,
            Err(CoreError::ClientAuthenticationFailed)
        ));
    }
}
