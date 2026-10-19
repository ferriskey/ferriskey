use std::time::Duration;

use tokio::sync::mpsc;

use crate::entities::CompassFlowStep;
use crate::ports::CompassFlowRepository;
use crate::recorder::CompassEvent;
use crate::value_objects::FlowCompletion;

#[derive(Debug, Clone, Copy)]
pub struct WriterConfig {
    pub linger: Duration,
    pub max_batch: usize,
}

impl Default for WriterConfig {
    fn default() -> Self {
        Self {
            linger: Duration::from_millis(100),
            max_batch: 256,
        }
    }
}

#[derive(Default)]
struct Batch {
    steps: Vec<CompassFlowStep>,
    completions: Vec<FlowCompletion>,
}

impl Batch {
    fn len(&self) -> usize {
        self.steps.len() + self.completions.len()
    }
}

pub async fn compass_writer_task<F: CompassFlowRepository>(
    mut receiver: mpsc::Receiver<CompassEvent>,
    flows: F,
    config: WriterConfig,
) {
    while let Some(event) = receiver.recv().await {
        let mut batch = Batch::default();
        absorb(event, &mut batch, &flows).await;

        let linger = tokio::time::sleep(config.linger);
        tokio::pin!(linger);

        let mut closed = false;
        while batch.len() < config.max_batch {
            tokio::select! {
                event = receiver.recv() => match event {
                    Some(event) => absorb(event, &mut batch, &flows).await,
                    None => {
                        closed = true;
                        break;
                    }
                },
                _ = &mut linger => break,
            }
        }

        flush(batch, &flows).await;

        if closed {
            return;
        }
    }
}

async fn absorb<F: CompassFlowRepository>(event: CompassEvent, batch: &mut Batch, flows: &F) {
    match event {
        CompassEvent::FlowStarted(flow) => {
            if let Err(e) = flows.create_flow(flow).await {
                tracing::error!("Compass writer: failed to persist flow: {e}");
            }
        }
        CompassEvent::StepRecorded(step) => batch.steps.push(step),
        CompassEvent::FlowCompleted(completion) => batch.completions.push(completion),
    }
}

async fn flush<F: CompassFlowRepository>(batch: Batch, flows: &F) {
    if batch.len() == 0 {
        return;
    }

    if let Err(e) = flows.write_batch(batch.steps, batch.completions).await {
        tracing::error!("Compass writer: failed to persist a batch: {e}");
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use chrono::Utc;
    use ferriskey_domain::common::app_errors::CoreError;
    use ferriskey_domain::realm::RealmId;
    use tokio::sync::mpsc;
    use uuid::Uuid;

    use crate::entities::{CompassFlow, CompassFlowStep, FlowId, FlowStatus, FlowStepName};
    use crate::ports::MockCompassFlowRepository;
    use crate::recorder::CompassEvent;
    use crate::value_objects::{FlowCompletion, StepOutcome};

    use super::{WriterConfig, compass_writer_task};

    fn step(flow_id: FlowId) -> CompassEvent {
        CompassEvent::StepRecorded(CompassFlowStep::new(
            flow_id,
            FlowStepName::CredentialValidation,
            StepOutcome::success(),
            Utc::now(),
        ))
    }

    fn completion(flow_id: FlowId) -> CompassEvent {
        CompassEvent::FlowCompleted(FlowCompletion {
            flow_id: flow_id.0,
            status: FlowStatus::Success,
            completed_at: Utc::now(),
            duration_ms: 12,
            user_id: Some(Uuid::new_v4()),
        })
    }

    fn config(linger_ms: u64, max_batch: usize) -> WriterConfig {
        WriterConfig {
            linger: Duration::from_millis(linger_ms),
            max_batch,
        }
    }

    fn flows_reporting_batches(
        fail_first: bool,
    ) -> (
        MockCompassFlowRepository,
        mpsc::UnboundedReceiver<(usize, usize)>,
    ) {
        let (seen_tx, seen_rx) = mpsc::unbounded_channel();
        let mut flows = MockCompassFlowRepository::new();
        let mut calls = 0;
        flows
            .expect_write_batch()
            .returning(move |steps, completions| {
                calls += 1;
                let _ = seen_tx.send((steps.len(), completions.len()));
                let failed = fail_first && calls == 1;
                Box::pin(async move {
                    if failed {
                        Err(CoreError::InternalServerError)
                    } else {
                        Ok(())
                    }
                })
            });
        (flows, seen_rx)
    }

    #[tokio::test(start_paused = true)]
    async fn the_events_of_one_cycle_are_written_as_one_batch() {
        let (tx, rx) = mpsc::channel(16);
        let (flows, mut seen) = flows_reporting_batches(false);
        let writer = tokio::spawn(compass_writer_task(rx, flows, config(100, 256)));
        let flow_id = FlowId::new();

        for _ in 0..3 {
            tx.send(step(flow_id.clone())).await.unwrap();
        }
        tx.send(completion(flow_id)).await.unwrap();
        drop(tx);

        writer.await.unwrap();
        assert_eq!(seen.recv().await, Some((3, 1)));
        assert!(seen.try_recv().is_err());
    }

    #[tokio::test(start_paused = true)]
    async fn a_batch_is_flushed_when_the_linger_elapses() {
        let (tx, rx) = mpsc::channel(16);
        let (flows, mut seen) = flows_reporting_batches(false);
        let _writer = tokio::spawn(compass_writer_task(rx, flows, config(100, 256)));
        let flow_id = FlowId::new();

        tx.send(step(flow_id.clone())).await.unwrap();
        assert_eq!(seen.recv().await, Some((1, 0)));

        tx.send(step(flow_id.clone())).await.unwrap();
        tx.send(step(flow_id)).await.unwrap();
        assert_eq!(seen.recv().await, Some((2, 0)));
    }

    #[tokio::test(start_paused = true)]
    async fn a_full_batch_is_flushed_before_the_linger() {
        let (tx, rx) = mpsc::channel(16);
        let (flows, mut seen) = flows_reporting_batches(false);
        let _writer = tokio::spawn(compass_writer_task(rx, flows, config(60_000, 2)));
        let flow_id = FlowId::new();

        tx.send(step(flow_id.clone())).await.unwrap();
        tx.send(step(flow_id)).await.unwrap();

        assert_eq!(seen.recv().await, Some((2, 0)));
    }

    #[tokio::test(start_paused = true)]
    async fn a_flow_start_is_written_at_once_outside_the_batch() {
        let (tx, rx) = mpsc::channel(16);
        let (stored_tx, mut stored_rx) = mpsc::unbounded_channel();
        let mut flows = MockCompassFlowRepository::new();
        flows.expect_create_flow().times(1).returning(move |flow| {
            let _ = stored_tx.send(flow.id);
            Box::pin(async { Ok(()) })
        });
        flows.expect_write_batch().never();
        let _writer = tokio::spawn(compass_writer_task(rx, flows, config(60_000, 256)));
        let flow = CompassFlow::new(
            RealmId::from(Uuid::new_v4()),
            Some("console".to_string()),
            "password".to_string(),
            None,
            None,
        );
        let flow_id = flow.id.clone();

        tx.send(CompassEvent::FlowStarted(flow)).await.unwrap();

        assert_eq!(stored_rx.recv().await, Some(flow_id));
    }

    #[tokio::test(start_paused = true)]
    async fn a_failed_batch_does_not_stop_the_writer() {
        let (tx, rx) = mpsc::channel(16);
        let (flows, mut seen) = flows_reporting_batches(true);
        let writer = tokio::spawn(compass_writer_task(rx, flows, config(100, 256)));
        let flow_id = FlowId::new();

        tx.send(step(flow_id.clone())).await.unwrap();
        assert_eq!(seen.recv().await, Some((1, 0)));

        tx.send(completion(flow_id)).await.unwrap();
        drop(tx);

        writer.await.unwrap();
        assert_eq!(seen.recv().await, Some((0, 1)));
    }
}
