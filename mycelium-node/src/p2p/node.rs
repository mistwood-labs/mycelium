use anyhow::Result;
use libp2p::futures::StreamExt;
use libp2p::gossipsub::{
    Gossipsub, GossipsubConfigBuilder, IdentTopic, MessageAuthenticity, ValidationMode,
};
use libp2p::{
    core::upgrade, identity, mplex, noise, swarm::Swarm, tcp::tokio::Transport as TcpTransport,
    yamux, Multiaddr, PeerId, Transport,
};
use std::time::Duration;

pub async fn start_node(_port: u16) -> Result<()> {
    // Generate identity keypair
    let id_keys = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(id_keys.public());
    println!("P2P node starting with PeerId: {}", peer_id);

    // Build transport
    let transport = TcpTransport::default()
        .upgrade(upgrade::Version::V1)
        .authenticate(noise::NoiseAuthenticated::xx(&id_keys).unwrap())
        .multiplex(yamux::YamuxConfig::default())
        .boxed();

    // Create Gossipsub
    let gossipsub_config = GossipsubConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(10))
        .validation_mode(ValidationMode::Strict)
        .build()
        .expect("Valid config");

    let mut behavior = Gossipsub::new(
        MessageAuthenticity::Signed(id_keys.clone()),
        gossipsub_config,
    )
    .expect("Correct config");

    // Join default topic
    let topic = IdentTopic::new("mycelium");
    behavior.subscribe(&topic)?;

    // Build swarm
    let mut swarm = Swarm::with_tokio_executor(transport, behavior, peer_id);

    // Main loop: not doing anything yet
    loop {
        tokio::select! {
            event = swarm.select_next_some() => {
                println!("P2P Event: {:?}", event);
            }
        }
    }
}
