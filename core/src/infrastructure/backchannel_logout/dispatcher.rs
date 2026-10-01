use std::sync::Arc;

use tokio::sync::{Semaphore, mpsc};
use tracing::warn;

use crate::domain::authentication::backchannel_logout::{
    BackchannelLogoutNotifier, BackchannelLogoutService, EndedSession,
};

const QUEUE_CAPACITY: usize = 1024;
const CONCURRENT_DELIVERIES: usize = 16;

/// Hands ended sessions to a background worker. Enqueuing never blocks: when
/// the queue is full the session is dropped with a warning, which costs the
/// clients a logout notice but never costs the user their logout.
#[derive(Debug, Clone)]
pub struct BackchannelLogoutDispatcher {
    sender: Option<mpsc::Sender<EndedSession>>,
}

impl BackchannelLogoutDispatcher {
    /// Starts the worker that delivers logout tokens through `service`.
    pub fn spawn<S>(service: Arc<S>) -> Self
    where
        S: BackchannelLogoutService + 'static,
    {
        let (sender, receiver) = mpsc::channel(QUEUE_CAPACITY);
        tokio::spawn(deliver_ended_sessions(receiver, service));
        Self {
            sender: Some(sender),
        }
    }

    /// A dispatcher that drops everything, for tests and tools that run
    /// without a worker.
    pub fn disabled() -> Self {
        Self { sender: None }
    }
}

impl BackchannelLogoutNotifier for BackchannelLogoutDispatcher {
    fn sessions_ended(&self, sessions: Vec<EndedSession>) {
        let Some(sender) = &self.sender else {
            return;
        };
        for session in sessions {
            if sender.try_send(session).is_err() {
                warn!(session_id = %session.session_id, "Back-channel logout queue full, dropping a session");
            }
        }
    }
}

async fn deliver_ended_sessions<S>(mut receiver: mpsc::Receiver<EndedSession>, service: Arc<S>)
where
    S: BackchannelLogoutService + 'static,
{
    let permits = Arc::new(Semaphore::new(CONCURRENT_DELIVERIES));
    while let Some(session) = receiver.recv().await {
        let Ok(permit) = permits.clone().acquire_owned().await else {
            return;
        };
        let service = service.clone();
        tokio::spawn(async move {
            service.deliver(session).await;
            drop(permit);
        });
    }
}
