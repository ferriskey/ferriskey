//! Adapters for OIDC Back-Channel Logout: who took part in a session, how a
//! logout token is signed and sent, and the hook that reports ended sessions.

pub mod dispatcher;
pub mod notifying_sessions;
pub mod participants;
pub mod sender;
pub mod signer;

pub use dispatcher::BackchannelLogoutDispatcher;
pub use notifying_sessions::NotifyingUserSessionRepository;
pub use participants::PostgresSessionParticipantRepository;
pub use sender::HttpLogoutTokenSender;
pub use signer::KeystoreLogoutTokenSigner;
