use ferriskey_compass::{
    ports::{CompassFlowRepository, CompassFlowStepRepository},
    recorder::CompassEvent,
};
use tokio::sync::mpsc;

pub async fn compass_writer_task<F, S>(
    mut receiver: mpsc::Receiver<CompassEvent>,
    flow_repo: F,
    step_repo: S,
) where
    F: CompassFlowRepository,
    S: CompassFlowStepRepository,
{
    while let Some(event) = receiver.recv().await {
        match event {
            CompassEvent::FlowStarted(flow) => {
                if let Err(e) = flow_repo.create_flow(flow).await {
                    tracing::error!("Compass writer: failed to persist flow: {e}");
                }
            }
            CompassEvent::StepRecorded(step) => {
                if let Err(e) = step_repo.create_step(step).await {
                    tracing::error!("Compass writer: failed to persist step: {e}");
                }
            }
            CompassEvent::FlowCompleted {
                flow_id,
                status,
                completed_at,
                duration_ms,
                user_id,
            } => {
                if let Err(e) = flow_repo
                    .update_flow_status(flow_id, status, completed_at, Some(duration_ms), user_id)
                    .await
                {
                    tracing::error!("Compass writer: failed to update flow status: {e}");
                }
            }
        }
    }
}
