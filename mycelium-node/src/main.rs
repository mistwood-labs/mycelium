mod config;
mod crypto;
mod grpc;
mod p2p;
mod storage;
mod types;

use tokio::try_join;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    // Load configuration
    let config = config::Config::load()?;

    // Start gRPC server
    let grpc_server = grpc::server::start_server(config.grpc_port);

    // Start P2P node
    let p2p_node = p2p::node::start_node(config.p2p_port);

    // Run both tasks concurrently
    try_join!(grpc_server, p2p_node,)?;

    Ok(())
}
