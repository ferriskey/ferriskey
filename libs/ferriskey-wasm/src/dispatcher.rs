use ferriskey_wasm_guest::ExtensionEvent;
use tokio::sync::mpsc;

/// A cloneable, Send+Sync handle for dispatching events to WASM extensions.
/// Modeled after FlowRecorder from ferriskey-compass.
#[derive(Clone, Debug)]
pub struct ExtensionDispatcher {
    sender: Option<mpsc::UnboundedSender<ExtensionEvent>>,
}

impl ExtensionDispatcher {
    pub fn new(sender: mpsc::UnboundedSender<ExtensionEvent>) -> Self {
        Self {
            sender: Some(sender),
        }
    }

    /// No-op dispatcher when extensions are disabled.
    pub fn disabled() -> Self {
        Self { sender: None }
    }

    pub fn dispatch(&self, event: ExtensionEvent) {
        if let Some(tx) = &self.sender {
            let _ = tx.send(event);
        }
    }
}
