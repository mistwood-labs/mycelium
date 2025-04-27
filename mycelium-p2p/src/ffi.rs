use crate::node;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use tokio::runtime::Runtime;

static RUNTIME: Lazy<Mutex<Option<Runtime>>> = Lazy::new(|| Mutex::new(None));

#[no_mangle]
pub extern "C" fn start_p2p_node() {
    let mut rt_guard = RUNTIME.lock().unwrap();
    if rt_guard.is_none() {
        let runtime = Runtime::new().unwrap();
        runtime.spawn(async {
            let _ = node::start().await;
        });
        *rt_guard = Some(runtime);
    }
}

#[no_mangle]
pub extern "C" fn stop_p2p_node() {
    let mut rt_guard = RUNTIME.lock().unwrap();
    if let Some(runtime) = rt_guard.take() {
        drop(runtime);
    }
}
