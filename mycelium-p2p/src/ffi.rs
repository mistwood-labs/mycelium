use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
    sync::{Mutex, OnceLock},
    thread::JoinHandle,
};

static NODE_THREAD: OnceLock<Mutex<Option<JoinHandle<()>>>> = OnceLock::new();

#[no_mangle]
pub extern "C" fn start_node(listen_addr: *const c_char) -> bool {
    let listen_addr = unsafe {
        if listen_addr.is_null() {
            eprintln!("Null listen_addr pointer");
            return false;
        }
        match CStr::from_ptr(listen_addr).to_str() {
            Ok(s) => s.to_owned(),
            Err(_) => {
                eprintln!("Failed to parse listen_addr");
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

    let handle = std::thread::spawn(move || {
        crate::node::MyNode::new()
            .expect("Failed to create node")
            .start(listen_addr);
    });

    *guard = Some(handle);

    true
}

#[no_mangle]
pub extern "C" fn mycelium_p2p_node_stop() -> bool {
    let mutex = NODE_THREAD.get_or_init(|| Mutex::new(None));
    let mut guard = mutex.lock().unwrap();

    if let Some(handle) = guard.take() {
        // Just detach for now
        handle.join().ok();
        println!("Node stopped");
        true
    } else {
        eprintln!("No node was running");
        false
    }
}

#[no_mangle]
pub extern "C" fn discovered_nodes() -> *mut c_char {
    let peers = crate::node::MY_NODE
        .lock()
        .unwrap()
        .discovered_nodes()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    let json = serde_json::to_string(&peers).unwrap();
    CString::new(json).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn connected_peers() -> *mut c_char {
    let peers = crate::node::MY_NODE
        .lock()
        .unwrap()
        .connected_peers()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    let json = serde_json::to_string(&peers).unwrap();
    CString::new(json).unwrap().into_raw()
}
