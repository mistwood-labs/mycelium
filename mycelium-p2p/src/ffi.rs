use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::c_char;
use std::sync::{Mutex, OnceLock};
use std::thread::{self, JoinHandle};

use crate::node;

static NODE_THREAD: OnceLock<Mutex<Option<JoinHandle<()>>>> = OnceLock::new();

#[no_mangle]
pub extern "C" fn mycelium_p2p_node_start(listen_addr: *const c_char) -> bool {
    let _listen_addr = unsafe {
        if listen_addr.is_null() {
            eprintln!("listen_addr pointer was null");
            return false;
        }
        match CStr::from_ptr(listen_addr).to_str() {
            Ok(s) => s.to_owned(),
            Err(_) => {
                eprintln!("failed to parse listen_addr");
                return false;
            }
        }
    };

    let mutex = NODE_THREAD.get_or_init(|| Mutex::new(None));
    let mut guard = mutex.lock().unwrap();

    if guard.is_some() {
        eprintln!("Node is already running");
        return false;
    }

    let handle = thread::spawn(move || {
        node::start_node();
    });

    *guard = Some(handle);
    true
}

#[no_mangle]
pub extern "C" fn mycelium_p2p_node_stop() -> bool {
    let mutex = NODE_THREAD.get_or_init(|| Mutex::new(None));
    let mut guard = mutex.lock().unwrap();

    if let Some(handle) = guard.take() {
        // 今は強制終了できないため、detachするのみ
        handle.join().ok();
        println!("Node stopped");
        true
    } else {
        eprintln!("No node was running");
        false
    }
}

/// Search for connected peers.
/// Returns a JSON string of PeerId list.
#[no_mangle]
pub extern "C" fn search_peers() -> *mut c_char {
    let peers = match crate::node::P2P_NODE.lock().unwrap().search_peers() {
        Ok(ids) => ids,
        Err(e) => {
            eprintln!("search_peers failed: {:?}", e);
            return std::ptr::null_mut();
        }
    };
    let json = serde_json::to_string(&peers).unwrap_or_else(|_| "[]".to_string());
    CString::new(json).unwrap().into_raw()
}
