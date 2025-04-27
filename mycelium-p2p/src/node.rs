use anyhow::Result;
use libp2p::{
    core::upgrade,
    dns,
    futures::StreamExt,
    gossipsub::{
        self, AllowAllSubscriptionFilter, Behaviour as GossipsubBehaviour, IdentTopic,
        IdentityTransform, MessageAuthenticity, ValidationMode,
    },
    identity, noise,
    swarm::SwarmBuilder,
    tcp, websocket, yamux, Multiaddr, PeerId, Transport,
};

pub async fn start() -> Result<()> {
    let id_keys = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(id_keys.public());
    println!("Local peer id: {:?}", peer_id);

    let transport = tcp::tokio::Transport::default()
        .upgrade(upgrade::Version::V1)
        .authenticate(noise::Config::new(&id_keys)?)
        .multiplex(yamux::Config::default())
        .boxed();

    let gossipsub_config = gossipsub::ConfigBuilder::default()
        .validation_mode(ValidationMode::Permissive)
        .build()
        .expect("Valid config");

    let behaviour = GossipsubBehaviour::<IdentityTransform, AllowAllSubscriptionFilter>::new(
        MessageAuthenticity::Signed(id_keys),
        gossipsub_config,
    )
    .expect("Correct config");

    let mut swarm = SwarmBuilder::with_tokio_executor(transport, behaviour, peer_id).build();

    loop {
        tokio::select! {
            event = swarm.select_next_some() => {
                println!("Swarm Event: {:?}", event);
            }
        }
    }
}
