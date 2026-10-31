use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use ferriskey_domain::realm::scope::Unscoped;
use ferriskey_domain::session::entities::UserSession;
use tokio::sync::{mpsc, watch};
use uuid::Uuid;

use super::subject::SubjectCaches;
use crate::infrastructure::common::TtlCache;

pub const LONGEST_TOKEN_STATE_TTL: Duration = Duration::from_secs(30);

#[derive(Debug)]
pub struct TokenState {
    sessions: TtlCache<Uuid, Option<Unscoped<UserSession>>>,
    unrevoked: TtlCache<String, bool>,
    subject: SubjectCaches,
    generation: watch::Sender<u64>,
    outbox: mpsc::UnboundedSender<()>,
    synced: AtomicBool,
}

#[derive(Debug)]
pub struct ChangeFeed(mpsc::UnboundedReceiver<()>);

impl ChangeFeed {
    pub async fn next(&mut self) -> bool {
        if self.0.recv().await.is_none() {
            return false;
        }
        while self.0.try_recv().is_ok() {}

        true
    }
}

impl TokenState {
    pub fn new(ttl: Duration) -> (Arc<Self>, ChangeFeed) {
        let ttl = ttl.min(LONGEST_TOKEN_STATE_TTL);
        let (outbox, feed) = mpsc::unbounded_channel();
        let state = Self {
            sessions: TtlCache::new(ttl),
            unrevoked: TtlCache::new(ttl),
            subject: SubjectCaches::new(ttl),
            generation: watch::channel(0).0,
            outbox,
            synced: AtomicBool::new(false),
        };

        (Arc::new(state), ChangeFeed(feed))
    }

    pub(super) fn sessions(&self) -> &TtlCache<Uuid, Option<Unscoped<UserSession>>> {
        &self.sessions
    }

    pub(super) fn unrevoked(&self) -> &TtlCache<String, bool> {
        &self.unrevoked
    }

    pub(super) fn subject(&self) -> &SubjectCaches {
        &self.subject
    }

    pub fn generation(&self) -> u64 {
        *self.generation.borrow()
    }

    pub fn subscribe(&self) -> watch::Receiver<u64> {
        self.generation.subscribe()
    }

    pub fn synced(&self) -> bool {
        self.synced.load(Ordering::SeqCst)
    }

    pub fn set_synced(&self, synced: bool) {
        self.synced.store(synced, Ordering::SeqCst);
    }

    pub fn changed(&self) {
        self.changed_elsewhere();
        let _ = self.outbox.send(());
    }

    pub fn changed_elsewhere(&self) {
        self.generation.send_modify(|generation| *generation += 1);
        self.sessions.clear();
        self.unrevoked.clear();
        self.subject.clear();
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    use super::TokenState;
    use crate::infrastructure::common::Retention;

    const TTL: Duration = Duration::from_secs(60);

    #[tokio::test]
    async fn a_local_change_bumps_the_generation_and_is_published_once_however_often_it_repeats() {
        let (state, mut feed) = TokenState::new(TTL);

        state.changed();
        state.changed();
        state.changed();

        assert_eq!(state.generation(), 3);
        assert!(feed.next().await);
        drop(state);
        assert!(!feed.next().await);
    }

    #[tokio::test]
    async fn a_change_seen_elsewhere_bumps_the_generation_without_publishing() {
        let (state, mut feed) = TokenState::new(TTL);

        state.changed_elsewhere();

        assert_eq!(state.generation(), 1);
        drop(state);
        assert!(!feed.next().await);
    }

    #[tokio::test]
    async fn a_change_empties_both_caches() {
        let (state, _feed) = TokenState::new(TTL);
        state
            .unrevoked()
            .load("hash", || async {
                Ok::<_, String>((false, Retention::Keep))
            })
            .await
            .expect("load");
        state
            .sessions()
            .load(&uuid::Uuid::nil(), || async {
                Ok::<_, String>((None, Retention::Keep))
            })
            .await
            .expect("load");

        state.changed();

        let loads = AtomicUsize::new(0);
        state
            .unrevoked()
            .load("hash", || async {
                loads.fetch_add(1, Ordering::SeqCst);
                Ok::<_, String>((false, Retention::Keep))
            })
            .await
            .expect("load");
        assert_eq!(loads.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn a_change_empties_the_subject_caches() {
        let (state, _feed) = TokenState::new(TTL);
        state.set_synced(true);
        let id = uuid::Uuid::nil();
        let loads = AtomicUsize::new(0);
        let read = || async {
            state
                .read_subject(
                    &state.subject().user_roles,
                    &id,
                    |_| true,
                    || async {
                        loads.fetch_add(1, Ordering::SeqCst);
                        Ok::<_, String>(Vec::new())
                    },
                )
                .await
                .expect("read")
        };

        read().await;
        read().await;
        assert_eq!(loads.load(Ordering::SeqCst), 1);

        state.changed_elsewhere();
        read().await;
        assert_eq!(loads.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn a_state_is_not_in_sync_until_the_listener_says_so() {
        let (state, _feed) = TokenState::new(TTL);
        assert!(!state.synced());

        state.set_synced(true);
        assert!(state.synced());

        state.set_synced(false);
        assert!(!state.synced());
    }
}
