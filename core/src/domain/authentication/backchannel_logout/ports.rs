use uuid::Uuid;

use crate::domain::authentication::backchannel_logout::entities::{
    DeliveryReport, EndedSession, LogoutDeliveryError, LogoutTokenClaims, SessionParticipant,
};
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::realm::entities::RealmId;

/// The clients that received a token in a session.
#[cfg_attr(test, mockall::automock)]
pub trait SessionParticipantRepository: Send + Sync {
    fn participants(
        &self,
        realm_id: RealmId,
        session_id: Uuid,
    ) -> impl Future<Output = Result<Vec<SessionParticipant>, CoreError>> + Send;
}

/// Signs logout token claims with the realm key, `typ: logout+jwt`.
#[cfg_attr(test, mockall::automock)]
pub trait LogoutTokenSigner: Send + Sync {
    fn sign(
        &self,
        realm_id: RealmId,
        claims: LogoutTokenClaims,
    ) -> impl Future<Output = Result<String, CoreError>> + Send;
}

/// POSTs one logout token to one client endpoint, once.
#[cfg_attr(test, mockall::automock)]
pub trait LogoutTokenSender: Send + Sync {
    fn send(
        &self,
        endpoint: String,
        logout_token: String,
    ) -> impl Future<Output = Result<(), LogoutDeliveryError>> + Send;
}

/// Told when sessions end. It must return at once: logging out never waits
/// for, nor fails because of, a client that is slow or down.
pub trait BackchannelLogoutNotifier: Send + Sync {
    fn sessions_ended(&self, sessions: Vec<EndedSession>);
}

pub trait BackchannelLogoutService: Send + Sync {
    /// Sends a logout token to every client of `session` that registered a
    /// back-channel logout endpoint.
    fn deliver(&self, session: EndedSession) -> impl Future<Output = DeliveryReport> + Send;
}
