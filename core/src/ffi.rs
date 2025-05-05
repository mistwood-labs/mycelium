use crate::{network::MyNode, proto};
use libc::{c_char, c_uchar};
use once_cell::sync::Lazy;
use prost::Message;
use std::{
    ffi::{CStr, CString},
    panic,
};
use tokio::sync::Mutex;

static NODE: Lazy<Mutex<MyNode>> = Lazy::new(|| {
    futures::executor::block_on(async {
        Mutex::new(MyNode::new().await.expect("NODE: failed to create"))
    })
});
static TOKIO_RT: Lazy<tokio::runtime::Runtime> = Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
});

/// # Safety
///
/// Safe unless...
#[no_mangle]
pub unsafe extern "C" fn node_start(addr: *const c_char) -> c_uchar {
    env_logger::init();
    log::debug!("env_logger initialized");

    let res = panic::catch_unwind(|| {
        let addr = unsafe { CStr::from_ptr(addr) }.to_str().unwrap_or_default();
        TOKIO_RT.block_on(async { NODE.lock().await.start(addr).await.is_ok() as u8 })
    });
    res.unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn node_stop() -> c_uchar {
    let res = panic::catch_unwind(|| {
        TOKIO_RT.block_on(async { NODE.lock().await.stop().await.is_ok() as u8 })
    });
    res.unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn connected_peers() -> *mut c_char {
    let res = panic::catch_unwind(|| {
        let peers: Vec<_> = TOKIO_RT.block_on(async {
            NODE.lock()
                .await
                .connected_peers()
                .await
                .map(ToString::to_string)
                .collect()
        });
        let json = serde_json::to_string(&peers).unwrap();
        CString::new(json).unwrap().into_raw()
    });
    res.unwrap_or(std::ptr::null_mut())
}

#[no_mangle]
pub extern "C" fn discovered_nodes() -> *mut c_char {
    let res = panic::catch_unwind(|| {
        let peers: Vec<_> = TOKIO_RT.block_on(async {
            NODE.lock()
                .await
                .discovered_nodes()
                .await
                .map(|peer| {
                    let s = peer.to_string();
                    log::debug!("discovered peer: {}", s);
                    s
                })
                .collect()
        });
        let json = serde_json::to_string(&peers).unwrap();
        CString::new(json).unwrap().into_raw()
    });
    res.unwrap_or_else(|e| {
        eprintln!("FFI discovered_nodes panic: {:?}", e);
        std::ptr::null_mut()
    })
}

/// # Safety
///
/// Safe unless...
#[no_mangle]
pub unsafe extern "C" fn connect_to_peer(addr: *const c_char) -> c_uchar {
    let res = panic::catch_unwind(|| {
        let addr = unsafe { CStr::from_ptr(addr) }.to_str().unwrap_or_default();
        TOKIO_RT.block_on(async { NODE.lock().await.connect_to_peer(addr).await.is_ok() as u8 })
    });
    res.unwrap_or(0)
}

/// # Safety
///
/// Safe unless...
#[no_mangle]
pub unsafe extern "C" fn publish_post(
    topic: *const c_char,
    payload_ptr: *const u8,
    len: usize,
) -> c_uchar {
    let res = panic::catch_unwind(|| {
        let topic = unsafe { CStr::from_ptr(topic) }
            .to_str()
            .unwrap_or_default();
        let payload = unsafe { std::slice::from_raw_parts(payload_ptr, len) };
        let post = proto::SignedPost::decode(payload).unwrap();
        TOKIO_RT.block_on(async { NODE.lock().await.publish_post(topic, post).await.is_ok() as u8 })
    });
    res.unwrap_or(0)
}

/// # Safety
///
/// Safe unless...
#[no_mangle]
pub unsafe extern "C" fn subscribe_topic(topic: *const c_char) -> c_uchar {
    let res = panic::catch_unwind(|| {
        let topic = unsafe { CStr::from_ptr(topic) }
            .to_str()
            .unwrap_or_default();
        TOKIO_RT.block_on(async { NODE.lock().await.subscribe_topic(topic).await.is_ok() as u8 })
    });
    res.unwrap_or(0)
}

/// # Safety
///
/// Safe unless...
#[no_mangle]
pub unsafe extern "C" fn send_reaction(
    peer: *const c_char,
    payload_ptr: *const u8,
    len: usize,
) -> c_uchar {
    let res = panic::catch_unwind(|| {
        let peer = unsafe { CStr::from_ptr(peer) }.to_str().unwrap_or_default();
        let payload = unsafe { std::slice::from_raw_parts(payload_ptr, len) };
        let reaction = proto::SignedReaction::decode(payload).unwrap();
        TOKIO_RT.block_on(async {
            NODE.lock()
                .await
                .send_reaction(peer, reaction)
                .await
                .is_ok() as u8
        })
    });
    res.unwrap_or(0)
}
