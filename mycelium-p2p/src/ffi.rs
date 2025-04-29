use crate::node::MyNode;
use once_cell::sync::Lazy;
use serde::Serialize;
use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
    ptr::null_mut,
    sync::{Mutex, OnceLock},
    thread::JoinHandle,
};

static NODE: Lazy<Mutex<MyNode>> =
    Lazy::new(|| Mutex::new(MyNode::new().expect("NODE: failed to create")));

static NODE_THREAD: OnceLock<Mutex<Option<JoinHandle<()>>>> = OnceLock::new();

#[no_mangle]
pub extern "C" fn mycelium_p2p_node_start(listen_addr: *const c_char) -> bool {
    let listen_addr = unsafe {
        if listen_addr.is_null() {
            eprintln!("start_node: listen_addr is null");
            return false;
        }
        let Ok(s) = CStr::from_ptr(listen_addr).to_str() else {
            eprintln!("start_node: failed to parse listen_addr");
            return false;
        };
        s.to_owned()
    };

    let mutex = NODE_THREAD.get_or_init(|| Mutex::new(None));
    let Ok(mut guard) = mutex.lock() else {
        eprintln!("start_node: failed to lock NODE_THREAD");
        return false;
    };

    if guard.is_some() {
        eprintln!("start_node: node is already running");
        return false;
    }

    let handle = std::thread::spawn(move || {
        NODE.lock().unwrap().start(listen_addr);
    });

    *guard = Some(handle);

    true
}

#[no_mangle]
pub extern "C" fn mycelium_p2p_node_stop() -> bool {
    let mutex = NODE_THREAD.get_or_init(|| Mutex::new(None));
    let Ok(mut guard) = mutex.lock() else {
        eprintln!("stop_node: failed to lock NODE_THREAD");
        return false;
    };

    if let Some(handle) = guard.take() {
        // Just detach for now
        handle.join().ok();
        println!("stop_node: node stopped");
        true
    } else {
        eprintln!("stop_node: no node is running");
        false
    }
}

#[no_mangle]
pub extern "C" fn mycelium_p2p_connect_to_peer(addr: *const c_char) -> bool {
    if addr.is_null() {
        eprintln!("connect_to_peer: null pointer");
        return false;
    }
    let s = unsafe {
        CStr::from_ptr(addr)
            .to_str()
            .map_err(|_| "invalid UTF-8 in address".to_string())
    };
    match s {
        Ok(addr_str) => with_node(|node| {
            node.connect_to_peer(addr_str);
            Ok(())
        }),
        Err(e) => {
            eprintln!("connect_to_peer: {}", e);
            false
        }
    }
}

#[no_mangle]
pub extern "C" fn mycelium_p2p_publish_message(
    topic: *const c_char,
    data: *const u8,
    len: usize,
) -> bool {
    if topic.is_null() {
        eprintln!("publish_message: null topic pointer");
        return false;
    }
    let t = unsafe {
        CStr::from_ptr(topic)
            .to_str()
            .map(|s| s.to_string())
            .map_err(|_| "invalid UTF-8 in topic".to_string())
    };
    if t.is_err() {
        eprintln!("publish_message: {}", t.unwrap_err());
        return false;
    }
    if data.is_null() {
        eprintln!("publish_message: null data pointer");
        return false;
    }
    // safety: caller holds valid data and len
    let payload = unsafe { std::slice::from_raw_parts(data, len).to_vec() };
    let topic_str = t.unwrap();
    with_node(|node| {
        node.publish_message(topic_str, payload);
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn mycelium_p2p_subscribe_topic(topic: *const c_char) -> bool {
    if topic.is_null() {
        eprintln!("subscribe_topic: null pointer");
        return false;
    }
    let s = unsafe {
        CStr::from_ptr(topic)
            .to_str()
            .map(|s| s.to_string())
            .map_err(|_| "invalid UTF-8 in topic".to_string())
    };
    match s {
        Ok(topic_str) => with_node(|node| {
            node.subscribe_topic(topic_str);
            Ok(())
        }),
        Err(e) => {
            eprintln!("subscribe_topic: {}", e);
            false
        }
    }
}

#[no_mangle]
pub extern "C" fn mycelium_p2p_send_reaction(
    peer: *const c_char,
    data_ptr: *const u8,
    data_len: usize,
) -> bool {
    if peer.is_null() {
        eprintln!("peer ptr null");
        return false;
    }
    let peer_str = unsafe {
        match CStr::from_ptr(peer).to_str() {
            Ok(s) => s,
            Err(_) => {
                eprintln!("invalid UTF-8 in peer");
                return false;
            }
        }
    };
    if data_ptr.is_null() {
        eprintln!("data ptr null");
        return false;
    }
    let data = unsafe { std::slice::from_raw_parts(data_ptr, data_len).to_vec() };
    with_node(|node| {
        node.send_reaction(peer_str, data);
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn mycelium_p2p_local_peer_id() -> *mut c_char {
    node_json(|node| node.local_peer_id().to_string())
}

#[no_mangle]
pub extern "C" fn mycelium_p2p_connected_peers() -> *mut c_char {
    node_json(|node| {
        node.connected_peers()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
    })
}

#[no_mangle]
pub extern "C" fn mycelium_p2p_discovered_nodes() -> *mut c_char {
    node_json(|node| {
        node.discovered_nodes()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
    })
}

#[no_mangle]
pub extern "C" fn mycelium_p2p_free_string(s: *mut c_char) {
    if !s.is_null() {
        let _ = unsafe { CString::from_raw(s) };
    }
}

fn with_node<F>(f: F) -> bool
where
    F: FnOnce(&mut MyNode) -> Result<(), String>,
{
    match NODE.lock() {
        Ok(mut guard) => match f(&mut *guard) {
            Ok(_) => true,
            Err(e) => {
                eprintln!("with_node: node action error: {}", e);
                false
            }
        },
        Err(e) => {
            eprintln!("with_node: failed to lock NODE: {}", e);
            false
        }
    }
}

fn node_json<T, F>(f: F) -> *mut c_char
where
    T: Serialize,
    F: FnOnce(&mut MyNode) -> T,
{
    let mut guard = match NODE.lock() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("node_json: failed to lock NODE: {e}");
            return null_mut();
        }
    };

    let data = f(&mut *guard);

    let json = match serde_json::to_string(&data) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("node_json: failed to serialize to JSON: {e}");
            return null_mut();
        }
    };

    let cstr = match CString::new(json) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("node_json: CString::new failed: {e}");
            return null_mut();
        }
    };

    cstr.into_raw()
}
