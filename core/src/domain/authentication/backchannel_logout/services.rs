use std::sync::Arc;

use chrono::Utc;
use futures::future::join_all;
use tracing::{debug, warn};

use crate::domain::authentication::backchannel_logout::entities::{
    BackchannelLogoutConfig, DeliveryReport, EndedSession, LogoutTokenClaims, SessionParticipant,
};
use crate::domain::authentication::backchannel_logout::ports::{
    BackchannelLogoutService, LogoutTokenSender, LogoutTokenSigner, SessionParticipantRepository,
};
use crate::domain::client::entities::Client;
use crate::domain::client::ports::ClientRepository;

#[derive(Clone, Debug)]
pub struct BackchannelLogoutServiceImpl<P, C, G, S>
where
    P: SessionParticipantRepository,
    C: ClientRepository,
    G: LogoutTokenSigner,
    S: LogoutTokenSender,
{
    pub(crate) participants: Arc<P>,
    pub(crate) clients: Arc<C>,
    pub(crate) signer: Arc<G>,
    pub(crate) sender: Arc<S>,
    pub(crate) config: BackchannelLogoutConfig,
}

impl<P, C, G, S> BackchannelLogoutServiceImpl<P, C, G, S>
where
    P: SessionParticipantRepository,
    C: ClientRepository,
    G: LogoutTokenSigner,
    S: LogoutTokenSender,
{
    pub fn new(
        participants: Arc<P>,
        clients: Arc<C>,
        signer: Arc<G>,
        sender: Arc<S>,
        config: BackchannelLogoutConfig,
    ) -> Self {
        Self {
            participants,
            clients,
            signer,
            sender,
            config,
        }
    }

    /// The participant's client and its endpoint, when it is an enabled client
    /// of the session's realm that registered one.
    async fn endpoint_of(
        &self,
        session: &EndedSession,
        participant: &SessionParticipant,
    ) -> Option<(Client, String)> {
        let client = self
            .clients
            .get_by_client_id(participant.client_id.clone(), session.realm_id)
            .await
            .ok()?
            .across_realms();

        if client.realm_id != session.realm_id || !client.enabled {
            return None;
        }

        let endpoint = client.backchannel_logout_uri.clone()?;
        Some((client, endpoint))
    }

    /// Signs and sends one logout token, retrying as configured. Returns
    /// whether the client acknowledged it.
    async fn notify(
        &self,
        session: &EndedSession,
        participant: SessionParticipant,
    ) -> Option<bool> {
        let (client, endpoint) = self.endpoint_of(session, &participant).await?;

        let claims = LogoutTokenClaims::new(
            &participant,
            session,
            Utc::now().timestamp(),
            self.config.token_lifetime,
        );
        let token = match self.signer.sign(session.realm_id, claims).await {
            Ok(token) => token,
            Err(err) => {
                warn!(client_id = %client.client_id, error = %err, "Could not sign a logout token");
                return Some(false);
            }
        };

        for (attempt, delay) in self.config.attempt_delays.iter().enumerate() {
            if !delay.is_zero() {
                tokio::time::sleep(*delay).await;
            }

            match self.sender.send(endpoint.clone(), token.clone()).await {
                Ok(()) => {
                    debug!(client_id = %client.client_id, attempt, "Logout token delivered");
                    return Some(true);
                }
                Err(err) if err.is_retryable() => {
                    warn!(client_id = %client.client_id, attempt, error = %err, "Logout token not delivered, retrying");
                }
                Err(err) => {
                    warn!(client_id = %client.client_id, attempt, error = %err, "Logout token refused");
                    return Some(false);
                }
            }
        }

        warn!(client_id = %client.client_id, session_id = %session.session_id, "Gave up delivering a logout token");
        Some(false)
    }
}

impl<P, C, G, S> BackchannelLogoutService for BackchannelLogoutServiceImpl<P, C, G, S>
where
    P: SessionParticipantRepository,
    C: ClientRepository,
    G: LogoutTokenSigner,
    S: LogoutTokenSender,
{
    async fn deliver(&self, session: EndedSession) -> DeliveryReport {
        let participants = match self
            .participants
            .participants(session.realm_id, session.session_id)
            .await
        {
            Ok(participants) => participants,
            Err(err) => {
                warn!(session_id = %session.session_id, error = %err, "Could not list the clients of an ended session");
                return DeliveryReport::default();
            }
        };

        let mut unique: Vec<SessionParticipant> = Vec::new();
        for participant in participants {
            if !unique
                .iter()
                .any(|seen| seen.client_id == participant.client_id)
            {
                unique.push(participant);
            }
        }

        let outcomes = join_all(unique.into_iter().map(|participant| {
            let client_id = participant.client_id.clone();
            async move { (client_id, self.notify(&session, participant).await) }
        }))
        .await;

        let mut report = DeliveryReport::default();
        for (client_id, outcome) in outcomes {
            match outcome {
                Some(true) => report.delivered.push(client_id),
                Some(false) => report.failed.push(client_id),
                None => {}
            }
        }
        report
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;
    use std::time::Duration;

    use uuid::Uuid;

    use super::*;
    use crate::domain::authentication::backchannel_logout::entities::{
        BACKCHANNEL_LOGOUT_EVENT, LogoutDeliveryError,
    };
    use crate::domain::authentication::backchannel_logout::ports::{
        MockLogoutTokenSender, MockLogoutTokenSigner, MockSessionParticipantRepository,
    };
    use crate::domain::client::ports::MockClientRepository;
    use crate::domain::common::entities::app_errors::CoreError;
    use crate::domain::realm::entities::{RealmId, Unscoped};

    const ISSUER: &str = "https://sso.example.com/realms/home";

    type TestService = BackchannelLogoutServiceImpl<
        MockSessionParticipantRepository,
        MockClientRepository,
        MockLogoutTokenSigner,
        MockLogoutTokenSender,
    >;

    fn session() -> EndedSession {
        EndedSession {
            realm_id: RealmId::default(),
            session_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
        }
    }

    fn client(realm_id: RealmId, client_id: &str, endpoint: Option<&str>) -> Client {
        let mut client = Client::from_realm_and_client_id(realm_id, client_id.to_string());
        client.backchannel_logout_uri = endpoint.map(str::to_string);
        client
    }

    fn participant(client_id: &str) -> SessionParticipant {
        SessionParticipant {
            client_id: client_id.to_string(),
            issuer: ISSUER.to_string(),
        }
    }

    struct Harness {
        participants: Vec<SessionParticipant>,
        clients: Vec<Client>,
        /// Per endpoint, the outcomes of successive attempts; the last repeats.
        outcomes: Vec<(String, Vec<Result<(), LogoutDeliveryError>>)>,
        signed: Arc<
            Mutex<
                Vec<crate::domain::authentication::backchannel_logout::entities::LogoutTokenClaims>,
            >,
        >,
        sent: Arc<Mutex<Vec<(String, String)>>>,
    }

    impl Harness {
        fn new(participants: Vec<SessionParticipant>, clients: Vec<Client>) -> Self {
            Self {
                participants,
                clients,
                outcomes: Vec::new(),
                signed: Arc::default(),
                sent: Arc::default(),
            }
        }

        fn answering(
            mut self,
            endpoint: &str,
            outcomes: Vec<Result<(), LogoutDeliveryError>>,
        ) -> Self {
            self.outcomes.push((endpoint.to_string(), outcomes));
            self
        }

        fn build(&self) -> TestService {
            let mut participants = MockSessionParticipantRepository::new();
            let listed = self.participants.clone();
            participants.expect_participants().returning(move |_, _| {
                let listed = listed.clone();
                Box::pin(async move { Ok(listed) })
            });

            let mut clients = MockClientRepository::new();
            let known = self.clients.clone();
            clients
                .expect_get_by_client_id()
                .returning(move |client_id, _| {
                    let found = known.iter().find(|c| c.client_id == client_id).cloned();
                    Box::pin(async move { found.map(Unscoped::new).ok_or(CoreError::NotFound) })
                });

            let mut signer = MockLogoutTokenSigner::new();
            let signed = self.signed.clone();
            signer.expect_sign().returning(move |_, claims| {
                let token = format!("token-for-{}", claims.aud);
                signed.lock().expect("signed lock").push(claims);
                Box::pin(async move { Ok(token) })
            });

            let mut sender = MockLogoutTokenSender::new();
            let sent = self.sent.clone();
            let outcomes = self.outcomes.clone();
            sender.expect_send().returning(move |endpoint, token| {
                let mut sent = sent.lock().expect("sent lock");
                sent.push((endpoint.clone(), token));
                let attempt = sent.iter().filter(|(e, _)| *e == endpoint).count() - 1;
                let script = outcomes
                    .iter()
                    .find(|(e, _)| *e == endpoint)
                    .map(|(_, script)| script.clone())
                    .unwrap_or_else(|| vec![Ok(())]);
                let outcome = script
                    .get(attempt)
                    .or(script.last())
                    .cloned()
                    .unwrap_or(Ok(()));
                Box::pin(async move { outcome })
            });

            BackchannelLogoutServiceImpl::new(
                Arc::new(participants),
                Arc::new(clients),
                Arc::new(signer),
                Arc::new(sender),
                BackchannelLogoutConfig {
                    attempt_delays: vec![Duration::ZERO; 3],
                    token_lifetime: Duration::from_secs(120),
                },
            )
        }

        fn attempts_to(&self, endpoint: &str) -> usize {
            self.sent
                .lock()
                .expect("sent lock")
                .iter()
                .filter(|(e, _)| e == endpoint)
                .count()
        }
    }

    #[tokio::test]
    async fn every_client_with_an_endpoint_gets_a_logout_token() {
        let ended = session();
        let harness = Harness::new(
            vec![participant("orders"), participant("billing")],
            vec![
                client(
                    ended.realm_id,
                    "orders",
                    Some("https://orders.example/logout"),
                ),
                client(
                    ended.realm_id,
                    "billing",
                    Some("https://billing.example/logout"),
                ),
            ],
        );

        let report = harness.build().deliver(ended).await;

        assert_eq!(report.failed, Vec::<String>::new());
        let mut delivered = report.delivered.clone();
        delivered.sort();
        assert_eq!(delivered, vec!["billing", "orders"]);
        assert_eq!(harness.attempts_to("https://orders.example/logout"), 1);
        assert!(harness.sent.lock().expect("sent lock").contains(&(
            "https://billing.example/logout".to_string(),
            "token-for-billing".to_string()
        )));
    }

    #[tokio::test]
    async fn the_logout_token_names_the_session_and_its_user() {
        let ended = session();
        let harness = Harness::new(
            vec![participant("orders")],
            vec![client(
                ended.realm_id,
                "orders",
                Some("https://orders.example/logout"),
            )],
        );

        harness.build().deliver(ended).await;

        let signed = harness.signed.lock().expect("signed lock");
        assert_eq!(signed.len(), 1);
        let claims = &signed[0];
        assert_eq!(claims.iss, ISSUER);
        assert_eq!(claims.aud, "orders");
        assert_eq!(claims.sub, ended.user_id);
        assert_eq!(claims.sid, ended.session_id);
        assert_eq!(claims.exp - claims.iat, 120);
        assert_eq!(
            claims.events,
            serde_json::json!({ BACKCHANNEL_LOGOUT_EVENT: {} })
        );
    }

    #[tokio::test]
    async fn clients_without_an_endpoint_are_skipped() {
        let ended = session();
        let harness = Harness::new(
            vec![participant("spa"), participant("orders")],
            vec![
                client(ended.realm_id, "spa", None),
                client(
                    ended.realm_id,
                    "orders",
                    Some("https://orders.example/logout"),
                ),
            ],
        );

        let report = harness.build().deliver(ended).await;

        assert_eq!(report.delivered, vec!["orders"]);
        assert!(report.failed.is_empty());
        assert_eq!(harness.signed.lock().expect("signed lock").len(), 1);
    }

    #[tokio::test]
    async fn a_client_listed_several_times_is_notified_once() {
        let ended = session();
        let harness = Harness::new(
            vec![participant("orders"), participant("orders")],
            vec![client(
                ended.realm_id,
                "orders",
                Some("https://orders.example/logout"),
            )],
        );

        harness.build().deliver(ended).await;

        assert_eq!(harness.attempts_to("https://orders.example/logout"), 1);
    }

    #[tokio::test]
    async fn disabled_clients_and_clients_of_another_realm_are_skipped() {
        let ended = session();
        let mut disabled = client(
            ended.realm_id,
            "disabled",
            Some("https://disabled.example/logout"),
        );
        disabled.enabled = false;
        let foreign = client(
            RealmId::default(),
            "foreign",
            Some("https://foreign.example/logout"),
        );
        let harness = Harness::new(
            vec![participant("disabled"), participant("foreign")],
            vec![disabled, foreign],
        );

        let report = harness.build().deliver(ended).await;

        assert_eq!(report, DeliveryReport::default());
        assert!(harness.sent.lock().expect("sent lock").is_empty());
    }

    #[tokio::test]
    async fn a_transient_failure_is_retried_until_it_succeeds() {
        let ended = session();
        let endpoint = "https://orders.example/logout";
        let harness = Harness::new(
            vec![participant("orders")],
            vec![client(ended.realm_id, "orders", Some(endpoint))],
        )
        .answering(
            endpoint,
            vec![
                Err(LogoutDeliveryError::Transport("timeout".into())),
                Err(LogoutDeliveryError::Rejected(503)),
                Ok(()),
            ],
        );

        let report = harness.build().deliver(ended).await;

        assert_eq!(report.delivered, vec!["orders"]);
        assert_eq!(harness.attempts_to(endpoint), 3);
    }

    #[tokio::test]
    async fn delivery_gives_up_after_the_configured_attempts() {
        let ended = session();
        let endpoint = "https://orders.example/logout";
        let harness = Harness::new(
            vec![participant("orders")],
            vec![client(ended.realm_id, "orders", Some(endpoint))],
        )
        .answering(
            endpoint,
            vec![Err(LogoutDeliveryError::Transport("down".into()))],
        );

        let report = harness.build().deliver(ended).await;

        assert_eq!(report.failed, vec!["orders"]);
        assert_eq!(harness.attempts_to(endpoint), 3);
    }

    #[tokio::test]
    async fn a_refusal_is_not_retried() {
        let ended = session();
        let endpoint = "https://orders.example/logout";
        let harness = Harness::new(
            vec![participant("orders")],
            vec![client(ended.realm_id, "orders", Some(endpoint))],
        )
        .answering(endpoint, vec![Err(LogoutDeliveryError::Rejected(400))]);

        let report = harness.build().deliver(ended).await;

        assert_eq!(report.failed, vec!["orders"]);
        assert_eq!(harness.attempts_to(endpoint), 1);
    }

    #[tokio::test]
    async fn one_failing_client_does_not_stop_the_others() {
        let ended = session();
        let harness = Harness::new(
            vec![participant("down"), participant("orders")],
            vec![
                client(ended.realm_id, "down", Some("https://down.example/logout")),
                client(
                    ended.realm_id,
                    "orders",
                    Some("https://orders.example/logout"),
                ),
            ],
        )
        .answering(
            "https://down.example/logout",
            vec![Err(LogoutDeliveryError::ForbiddenAddress)],
        );

        let report = harness.build().deliver(ended).await;

        assert_eq!(report.delivered, vec!["orders"]);
        assert_eq!(report.failed, vec!["down"]);
    }
}
