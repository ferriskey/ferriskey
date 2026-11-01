use std::sync::Arc;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::authentication::pushed_authorization::entities::{
    AuthorizationParameters, PUSHED_REQUEST_LIFETIME_SECS, PushedAuthorizationRequest,
    PushedAuthorizationResponse, RequestUri,
};
use crate::domain::authentication::pushed_authorization::ports::PushedAuthorizationRequestRepository;
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::realm::entities::RealmId;

#[derive(Debug)]
pub struct PushedAuthorizationServiceImpl<S> {
    store: Arc<S>,
}

impl<S> Clone for PushedAuthorizationServiceImpl<S> {
    fn clone(&self) -> Self {
        Self {
            store: Arc::clone(&self.store),
        }
    }
}

impl<S: PushedAuthorizationRequestRepository> PushedAuthorizationServiceImpl<S> {
    pub fn new(store: Arc<S>) -> Self {
        Self { store }
    }

    /// Stores already validated parameters and returns the `request_uri` the
    /// client sends to `/auth`.
    pub async fn push(
        &self,
        realm_id: RealmId,
        client_id: Uuid,
        parameters: AuthorizationParameters,
        now: DateTime<Utc>,
    ) -> Result<PushedAuthorizationResponse, CoreError> {
        let request_uri = RequestUri::generate();
        let request =
            PushedAuthorizationRequest::new(&request_uri, realm_id, client_id, parameters, now);

        self.store.store(&request).await?;

        Ok(PushedAuthorizationResponse {
            request_uri: request_uri.as_str().to_string(),
            expires_in: PUSHED_REQUEST_LIFETIME_SECS,
        })
    }

    /// Consumes the request behind `request_uri`. It is refused when it is
    /// malformed, unknown, expired, already used or pushed by another client.
    pub async fn take(
        &self,
        realm_id: RealmId,
        client_id: Uuid,
        request_uri: &str,
        now: DateTime<Utc>,
    ) -> Result<AuthorizationParameters, CoreError> {
        let request_uri = RequestUri::parse(request_uri).ok_or(CoreError::InvalidRequestUri)?;

        self.store
            .take(realm_id, client_id, &request_uri.digest(), now)
            .await?
            .ok_or(CoreError::InvalidRequestUri)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::collections::HashMap;
    use std::sync::Mutex;

    use chrono::{DateTime, Duration, Utc};
    use uuid::Uuid;

    use super::PushedAuthorizationServiceImpl;
    use crate::domain::authentication::pushed_authorization::entities::{
        AuthorizationParameters, PUSHED_REQUEST_LIFETIME_SECS, PushedAuthorizationRequest,
        REQUEST_URI_PREFIX, RequestUri, RequestUriDigest,
    };
    use crate::domain::authentication::pushed_authorization::ports::PushedAuthorizationRequestRepository;
    use crate::domain::common::entities::app_errors::CoreError;
    use crate::domain::realm::entities::RealmId;

    #[derive(Debug, Default)]
    pub(crate) struct InMemoryPushedAuthorizationRequests(
        Mutex<HashMap<RequestUriDigest, PushedAuthorizationRequest>>,
    );

    impl PushedAuthorizationRequestRepository for InMemoryPushedAuthorizationRequests {
        async fn store(&self, request: &PushedAuthorizationRequest) -> Result<(), CoreError> {
            self.0
                .lock()
                .expect("store lock")
                .insert(request.digest.clone(), request.clone());
            Ok(())
        }

        async fn take(
            &self,
            realm_id: RealmId,
            client_id: Uuid,
            digest: &RequestUriDigest,
            now: DateTime<Utc>,
        ) -> Result<Option<AuthorizationParameters>, CoreError> {
            let mut requests = self.0.lock().expect("store lock");
            let usable = requests.get(digest).is_some_and(|request| {
                request.realm_id == realm_id
                    && request.client_id == client_id
                    && request.expires_at > now
            });
            Ok(usable
                .then(|| requests.remove(digest))
                .flatten()
                .map(|request| request.parameters))
        }

        async fn delete_expired(
            &self,
            before: DateTime<Utc>,
            limit: u64,
        ) -> Result<u64, CoreError> {
            let mut requests = self.0.lock().expect("store lock");
            let expired: Vec<RequestUriDigest> = requests
                .values()
                .filter(|request| request.expires_at < before)
                .take(limit as usize)
                .map(|request| request.digest.clone())
                .collect();
            for digest in &expired {
                requests.remove(digest);
            }
            Ok(expired.len() as u64)
        }
    }

    pub(crate) fn parameters() -> AuthorizationParameters {
        AuthorizationParameters {
            response_type: "code".to_string(),
            redirect_uri: "https://app.example/cb".to_string(),
            scope: Some("openid".to_string()),
            state: Some("xyz".to_string()),
            nonce: None,
            code_challenge: Some("E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM".to_string()),
            code_challenge_method: None,
            prompt: Some("login".to_string()),
            max_age: Some(300),
            resource: None,
        }
    }

    /// The behaviour every adapter of the port must have.
    pub(crate) async fn repository_contract<S: PushedAuthorizationRequestRepository>(
        store: &S,
        realm_id: RealmId,
        client_id: Uuid,
        other_client_id: Uuid,
    ) {
        let now = Utc::now();
        let push = |at: DateTime<Utc>| {
            let request_uri = RequestUri::generate();
            let request = PushedAuthorizationRequest::new(
                &request_uri,
                realm_id,
                client_id,
                parameters(),
                at,
            );
            (request_uri.digest(), request)
        };

        let (digest, request) = push(now);
        store.store(&request).await.expect("store");
        assert_eq!(
            store
                .take(realm_id, other_client_id, &digest, now)
                .await
                .expect("take"),
            None,
            "another client cannot use the request"
        );
        assert_eq!(
            store
                .take(RealmId::from(Uuid::new_v4()), client_id, &digest, now)
                .await
                .expect("take"),
            None,
            "another realm cannot use the request"
        );
        assert_eq!(
            store
                .take(realm_id, client_id, &digest, now)
                .await
                .expect("take"),
            Some(parameters()),
            "a refused attempt does not consume the request"
        );
        assert_eq!(
            store
                .take(realm_id, client_id, &digest, now)
                .await
                .expect("take"),
            None,
            "the request is single use"
        );

        let (digest, request) = push(now);
        store.store(&request).await.expect("store");
        let after_expiry = now + Duration::seconds(PUSHED_REQUEST_LIFETIME_SECS + 1);
        assert_eq!(
            store
                .take(realm_id, client_id, &digest, after_expiry)
                .await
                .expect("take"),
            None,
            "an expired request is refused"
        );

        let (live_digest, live) = push(now);
        store.store(&live).await.expect("store");
        let (_, stale) = push(now - Duration::seconds(PUSHED_REQUEST_LIFETIME_SECS * 10));
        store.store(&stale).await.expect("store");
        assert_eq!(
            store.delete_expired(now, 100).await.expect("purge"),
            1,
            "only the stale request is purged"
        );
        assert_eq!(
            store
                .take(realm_id, client_id, &live_digest, now)
                .await
                .expect("take"),
            Some(parameters()),
            "the purge keeps live requests"
        );
    }

    #[tokio::test]
    async fn the_in_memory_store_honours_the_contract() {
        repository_contract(
            &InMemoryPushedAuthorizationRequests::default(),
            RealmId::from(Uuid::new_v4()),
            Uuid::new_v4(),
            Uuid::new_v4(),
        )
        .await;
    }

    fn service() -> PushedAuthorizationServiceImpl<InMemoryPushedAuthorizationRequests> {
        PushedAuthorizationServiceImpl::new(std::sync::Arc::new(
            InMemoryPushedAuthorizationRequests::default(),
        ))
    }

    #[tokio::test]
    async fn a_pushed_request_is_replayed_once_for_its_client() {
        let service = service();
        let realm_id = RealmId::from(Uuid::new_v4());
        let client_id = Uuid::new_v4();
        let now = Utc::now();

        let response = service
            .push(realm_id, client_id, parameters(), now)
            .await
            .expect("push");

        assert!(response.request_uri.starts_with(REQUEST_URI_PREFIX));
        assert_eq!(response.expires_in, PUSHED_REQUEST_LIFETIME_SECS);
        assert_eq!(
            service
                .take(realm_id, client_id, &response.request_uri, now)
                .await
                .expect("take"),
            parameters()
        );
        assert!(matches!(
            service
                .take(realm_id, client_id, &response.request_uri, now)
                .await,
            Err(CoreError::InvalidRequestUri)
        ));
    }

    #[tokio::test]
    async fn a_request_uri_of_another_client_is_refused() {
        let service = service();
        let realm_id = RealmId::from(Uuid::new_v4());
        let now = Utc::now();
        let response = service
            .push(realm_id, Uuid::new_v4(), parameters(), now)
            .await
            .expect("push");

        assert!(matches!(
            service
                .take(realm_id, Uuid::new_v4(), &response.request_uri, now)
                .await,
            Err(CoreError::InvalidRequestUri)
        ));
    }

    #[tokio::test]
    async fn a_malformed_or_unknown_request_uri_is_refused() {
        let service = service();
        let realm_id = RealmId::from(Uuid::new_v4());

        for request_uri in [
            "",
            "https://evil.example/request.jwt",
            REQUEST_URI_PREFIX,
            RequestUri::generate().as_str(),
        ] {
            assert!(
                matches!(
                    service
                        .take(realm_id, Uuid::new_v4(), request_uri, Utc::now())
                        .await,
                    Err(CoreError::InvalidRequestUri)
                ),
                "{request_uri:?}"
            );
        }
    }

    #[tokio::test]
    async fn a_request_uri_is_refused_once_expired() {
        let service = service();
        let realm_id = RealmId::from(Uuid::new_v4());
        let client_id = Uuid::new_v4();
        let pushed_at = Utc::now();
        let response = service
            .push(realm_id, client_id, parameters(), pushed_at)
            .await
            .expect("push");

        assert!(matches!(
            service
                .take(
                    realm_id,
                    client_id,
                    &response.request_uri,
                    pushed_at + Duration::seconds(PUSHED_REQUEST_LIFETIME_SECS),
                )
                .await,
            Err(CoreError::InvalidRequestUri)
        ));
    }
}
