use std::path::Path;

use anyhow::Result;
use ferriskey_wasm_guest::ExtensionEvent;
use tokio::sync::mpsc;
use wasmtime::{Engine, Linker};
use wasmtime_wasi::preview1::WasiP1Ctx;

use crate::extension::WasmExtension;

pub struct ExtensionRuntime {
    senders: Vec<mpsc::UnboundedSender<ExtensionEvent>>,
}

impl ExtensionRuntime {
    pub fn load_from_directory(dir: &Path) -> Result<Self> {
        let mut senders = Vec::new();

        let entries = std::fs::read_dir(dir)?;

        for entry in entries {
            let path = entry?.path();
            if path.extension().is_some_and(|e| e == "wasm") {
                let (tx, rx) = mpsc::unbounded_channel::<ExtensionEvent>();
                let path_clone = path.clone();

                std::thread::spawn(move || {
                    Self::run_extension(path_clone, rx);
                });

                tracing::info!("Loading WASM extension: {}", path.display());
                senders.push(tx);
            }
        }

        tracing::info!("Loaded {} WASM extension(s)", senders.len());
        Ok(Self { senders })
    }

    fn run_extension(path: std::path::PathBuf, mut rx: mpsc::UnboundedReceiver<ExtensionEvent>) {
        let engine = Engine::default();
        let mut linker: Linker<WasiP1Ctx> = Linker::new(&engine);

        if let Err(e) = wasmtime_wasi::preview1::add_to_linker_sync(&mut linker, |ctx| ctx) {
            tracing::error!("Failed to add WASI to linker: {}", e);
            return;
        }

        let mut ext = match WasmExtension::load(&engine, &linker, &path) {
            Ok(ext) => ext,
            Err(e) => {
                tracing::error!("Failed to load extension {}: {}", path.display(), e);
                return;
            }
        };

        if let Err(e) = ext.boot() {
            tracing::error!("Failed to boot extension {}: {}", ext.name(), e);
            return;
        }

        match ext.ready() {
            Ok(true) => tracing::info!("Extension '{}' is ready", ext.name()),
            Ok(false) => {
                tracing::error!("Extension '{}' is not ready after boot", ext.name());
                return;
            }
            Err(e) => {
                tracing::error!("Failed to check readiness of '{}': {}", ext.name(), e);
                return;
            }
        }

        while let Some(event) = rx.blocking_recv() {
            if let Err(e) = ext.emit(&event) {
                tracing::error!("Extension '{}' error on event: {}", ext.name(), e);
            }
        }

        tracing::info!("Extension '{}' channel closed, shutting down", ext.name());
    }

    pub fn dispatch(&self, event: ExtensionEvent) {
        for sender in &self.senders {
            let _ = sender.send(event.clone());
        }
    }
}
