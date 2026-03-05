use chrono::{DateTime, Utc};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::domain::compass::entities::{
    CompassFlow, CompassFlowStep, FlowId, FlowStatus, FlowStepName, StepStatus,
};
use crate::domain::realm::entities::RealmId;

pub enum CompassEvent {
    FlowStarted(CompassFlow),
    StepRecorded(CompassFlowStep),
    FlowCompleted {
        flow_id: Uuid,
        status: FlowStatus,
        completed_at: DateTime<Utc>,
        duration_ms: i64,
        user_id: Option<Uuid>,
    },
}

#[derive(Clone, Debug)]
pub struct FlowRecorder {
    enabled: bool,
    sender: Option<mpsc::Sender<CompassEvent>>,
}

impl FlowRecorder {
    pub fn new(sender: mpsc::Sender<CompassEvent>) -> Self {
        Self {
            enabled: true,
            sender: Some(sender),
        }
    }

    pub fn disabled() -> Self {
        Self {
            enabled: false,
            sender: None,
        }
    }

    fn send(&self, event: CompassEvent) {
        if !self.enabled {
            return;
        }
        if let Some(tx) = &self.sender
            && tx.try_send(event).is_err()
        {
            tracing::warn!("Compass: channel full, dropping event");
        }
    }

    pub fn start_flow(
        &self,
        realm_id: RealmId,
        client_id: Option<String>,
        grant_type: String,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> FlowId {
        let flow = CompassFlow::new(realm_id, client_id, grant_type, ip_address, user_agent);
        let id = flow.id.clone();
        self.send(CompassEvent::FlowStarted(flow));
        id
    }

    pub fn record_step(
        &self,
        flow_id: FlowId,
        step_name: FlowStepName,
        status: StepStatus,
        duration_ms: Option<i64>,
        error_code: Option<String>,
        error_message: Option<String>,
    ) {
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
        flow_id: FlowId,
        status: FlowStatus,
        duration_ms: i64,
        user_id: Option<Uuid>,
    ) {
        self.send(CompassEvent::FlowCompleted {
            flow_id: flow_id.0,
            status,
            completed_at: Utc::now(),
            duration_ms,
            user_id,
        });
    }
}
