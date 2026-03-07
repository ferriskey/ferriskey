use std::path::Path;

use anyhow::{Context, Result};
use ferriskey_wasm_guest::ExtensionEvent;
use wasmtime::{Engine, Instance, Linker, Module, Store};
use wasmtime_wasi::WasiCtxBuilder;
use wasmtime_wasi::preview1::WasiP1Ctx;

pub struct WasmExtension {
    name: String,
    store: Store<WasiP1Ctx>,
    instance: Instance,
}

impl WasmExtension {
    pub fn load(engine: &Engine, linker: &Linker<WasiP1Ctx>, path: &Path) -> Result<Self> {
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let wasi_ctx = WasiCtxBuilder::new().inherit_stdio().build_p1();
        let mut store = Store::new(engine, wasi_ctx);
        let module = Module::from_file(engine, path)?;
        let instance = linker.instantiate(&mut store, &module)?;

        Ok(Self {
            name,
            store,
            instance,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn boot(&mut self) -> Result<()> {
        let boot_fn = self
            .instance
            .get_typed_func::<(), ()>(&mut self.store, "boot")?;
        boot_fn.call(&mut self.store, ())?;
        Ok(())
    }

    pub fn ready(&mut self) -> Result<bool> {
        let ready_fn = self
            .instance
            .get_typed_func::<(), u32>(&mut self.store, "ready")?;
        let result = ready_fn.call(&mut self.store, ())?;
        Ok(result != 0)
    }

    pub fn emit(&mut self, event: &ExtensionEvent) -> Result<()> {
        let json_bytes = serde_json::to_vec(event)?;
        let len = json_bytes.len() as u32;

        let alloc_fn = self
            .instance
            .get_typed_func::<u32, u32>(&mut self.store, "alloc")?;
        let ptr = alloc_fn.call(&mut self.store, len)?;

        let memory = self
            .instance
            .get_memory(&mut self.store, "memory")
            .context("missing memory export")?;
        memory.write(&mut self.store, ptr as usize, &json_bytes)?;

        let on_event_fn = self
            .instance
            .get_typed_func::<(u32, u32), ()>(&mut self.store, "on_event")?;
        on_event_fn.call(&mut self.store, (ptr, len))?;

        Ok(())
    }
}
