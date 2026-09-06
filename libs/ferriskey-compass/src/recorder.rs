use chrono::{DateTime, Utc};
use ferriskey_domain::realm::Realm;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

use crate::entities::{CompassFlow, CompassFlowStep, FlowId, FlowStatus, FlowStepName, StepStatus};

pub enum CompassEvent {
    FlowStarted {
        flow: CompassFlow,
        ack: oneshot::Sender<()>,
    },
    StepRecorded(CompassFlowStep),
    FlowCompleted {
        flow_id: Uuid,
        status: FlowStatus,
        completed_at: DateTime<Utc>,
        duration_ms: i64,
        user_id: Option<Uuid>,
    },
}

/// Records authentication flows for the Compass dashboard.
///
/// Tracing is opt-out per realm through `realm_settings.compass_enabled`. The
/// setting is read from the [`Realm`] handed to [`FlowRecorder::start_flow`],
/// which is the only way to obtain a [`FlowId`]: a caller cannot record steps
/// for a realm it never started a flow on, so the toggle cannot be bypassed by
/// reaching for the wrong method.
#[derive(Clone, Debug)]
pub struct FlowRecorder {
    sender: Option<mpsc::Sender<CompassEvent>>,
}

impl FlowRecorder {
    pub fn new(sender: mpsc::Sender<CompassEvent>) -> Self {
        Self {
            sender: Some(sender),
        }
    }

    /// A recorder that drops everything, for deployments and tests running
    /// without a Compass writer behind them.
    pub fn disabled() -> Self {
        Self { sender: None }
    }

    fn send(&self, event: CompassEvent) {
        if let Some(tx) = &self.sender
            && tx.try_send(event).is_err()
        {
            tracing::warn!("Compass: channel full, dropping event");
        }
    }

    /// Starts a flow, unless Compass is disabled for this realm.
    ///
    /// `None` means "not being traced". Every later call keyed on that `None`
    /// is a no-op, so a disabled realm cannot emit steps referencing a flow row
    /// that was never written.
    pub async fn start_flow(
        &self,
        realm: &Realm,
        client_id: Option<String>,
        grant_type: String,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Option<FlowId> {
        if !Self::is_enabled_for(realm) {
            return None;
        }

        let flow = CompassFlow::new(realm.id, client_id, grant_type, ip_address, user_agent);
        let id = flow.id.clone();

        let (ack_tx, ack_rx) = oneshot::channel();
        self.send(CompassEvent::FlowStarted { flow, ack: ack_tx });

        let _ = ack_rx.await;

        Some(id)
    }

    /// Realms whose settings row predates the toggle (migration
    /// `20260306000001`) have no stored preference; Compass ships on, so the
    /// absence of a row keeps tracing rather than silently stopping it.
    fn is_enabled_for(realm: &Realm) -> bool {
        realm
            .settings
            .as_ref()
            .map(|settings| settings.compass_enabled)
            .unwrap_or(true)
    }

    pub fn record_step(
        &self,
        flow_id: Option<FlowId>,
        step_name: FlowStepName,
        status: StepStatus,
        duration_ms: Option<i64>,
        error_code: Option<String>,
        error_message: Option<String>,
    ) {
        let Some(flow_id) = flow_id else {
            return;
        };

        let step = CompassFlowStep::new(
            flow_id,
            step_name,
            status,
            duration_ms,
            error_code,
            error_message,
            Utc::now(),
        );
        self.send(CompassEvent::StepRecorded(step));
    }

    pub fn complete_flow(
        &self,
        flow_id: Option<FlowId>,
        status: FlowStatus,
        duration_ms: i64,
        user_id: Option<Uuid>,
    ) {
        let Some(flow_id) = flow_id else {
            return;
        };

        self.send(CompassEvent::FlowCompleted {
            flow_id: flow_id.0,
            status,
            completed_at: Utc::now(),
            duration_ms,
            user_id,
        });
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use ferriskey_domain::realm::{Realm, RealmId, RealmSetting};
    use uuid::Uuid;

    use super::*;

    fn realm_with_compass(enabled: Option<bool>) -> Realm {
        let id = RealmId::from(Uuid::new_v4());

        Realm {
            id,
            name: "master".to_string(),
            display_name: None,
            settings: enabled.map(|compass_enabled| {
                let mut settings = RealmSetting::new(id, Some("RS256".to_string()));
                settings.compass_enabled = compass_enabled;
                settings
            }),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// `start_flow` waits for the writer to acknowledge the new flow, so a test
    /// that starts one needs something playing the writer's part. Returns the
    /// flow the recorder emitted, or `None` if it emitted nothing.
    fn stand_in_writer(
        mut events: mpsc::Receiver<CompassEvent>,
    ) -> tokio::task::JoinHandle<Option<CompassFlow>> {
        tokio::spawn(async move {
            match events.recv().await {
                Some(CompassEvent::FlowStarted { flow, ack }) => {
                    let _ = ack.send(());
                    Some(flow)
                }
                _ => None,
            }
        })
    }

    #[tokio::test]
    async fn a_realm_with_compass_disabled_records_nothing() {
        let (tx, mut events) = mpsc::channel(16);
        let recorder = FlowRecorder::new(tx);

        let flow_id = recorder
            .start_flow(
                &realm_with_compass(Some(false)),
                Some("account".to_string()),
                "authorization_code".to_string(),
                None,
                None,
            )
            .await;

        assert!(
            flow_id.is_none(),
            "a realm that switched Compass off must not open a flow"
        );
        assert!(
            events.try_recv().is_err(),
            "no event may reach the writer for a disabled realm"
        );
    }

    #[tokio::test]
    async fn a_realm_with_compass_enabled_opens_a_flow() {
        let (tx, events) = mpsc::channel(16);
        let recorder = FlowRecorder::new(tx);
        let realm = realm_with_compass(Some(true));
        let writer = stand_in_writer(events);

        let flow_id = recorder
            .start_flow(
                &realm,
                Some("account".to_string()),
                "authorization_code".to_string(),
                None,
                None,
            )
            .await
            .expect("an enabled realm opens a flow");

        let flow = writer
            .await
            .expect("writer task")
            .expect("a FlowStarted event");

        assert_eq!(flow.id, flow_id);
        assert_eq!(flow.realm_id, realm.id);
        assert_eq!(flow.status, FlowStatus::Pending);
    }

    /// Realms whose settings row predates migration `20260306000001` carry no
    /// preference. Compass ships on, so tracing continues rather than silently
    /// stopping for every realm created before the toggle existed.
    #[tokio::test]
    async fn a_realm_without_settings_keeps_tracing() {
        let (tx, events) = mpsc::channel(16);
        let recorder = FlowRecorder::new(tx);
        let writer = stand_in_writer(events);

        let flow_id = recorder
            .start_flow(
                &realm_with_compass(None),
                None,
                "password".to_string(),
                None,
                None,
            )
            .await;

        assert!(flow_id.is_some());
        assert!(writer.await.expect("writer task").is_some());
    }

    /// The disabled path must stay consistent end to end: a step keyed on a flow
    /// that was never opened would reference a row that does not exist.
    #[tokio::test]
    async fn steps_and_completion_are_no_ops_without_a_flow() {
        let (tx, mut events) = mpsc::channel(16);
        let recorder = FlowRecorder::new(tx);

        recorder.record_step(
            None,
            FlowStepName::Authorize,
            StepStatus::Success,
            None,
            None,
            None,
        );
        recorder.complete_flow(None, FlowStatus::Success, 12, None);

        assert!(
            events.try_recv().is_err(),
            "a step without a flow must not reach the writer"
        );
    }
}
