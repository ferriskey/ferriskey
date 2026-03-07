use std::cell::RefCell;

use ferriskey_wasm_guest::ExtensionEvent;

thread_local! {
    static READY: RefCell<bool> = const { RefCell::new(false) };
}

#[unsafe(no_mangle)]
pub extern "C" fn alloc(size: u32) -> u32 {
    let mut buf: Vec<u8> = Vec::with_capacity(size as usize);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr as u32
}

#[unsafe(no_mangle)]
pub extern "C" fn boot() {
    READY.with(|r| *r.borrow_mut() = true);
    println!("[Logger Extension] Booted!");
}

#[unsafe(no_mangle)]
pub extern "C" fn ready() -> u32 {
    READY.with(|r| *r.borrow() as u32)
}

#[unsafe(no_mangle)]
pub extern "C" fn on_event(ptr: u32, len: u32) {
    let slice = unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize) };

    match serde_json::from_slice::<ExtensionEvent>(slice) {
        Ok(event) => {
            println!(
                "[Logger Extension] event_type={} realm_id={} resource_id={}",
                event.event_type, event.realm_id, event.resource_id
            );
        }
        Err(err) => {
            eprintln!("[Logger Extension] Failed to deserialize event: {}", err);
        }
    }
}
