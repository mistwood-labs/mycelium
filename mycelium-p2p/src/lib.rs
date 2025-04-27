pub mod ffi;
pub mod node;

#[no_mangle]
pub extern "C" fn start_node() {
    node::start_node();
}
