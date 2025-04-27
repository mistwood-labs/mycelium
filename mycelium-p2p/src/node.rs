use libp2p::{
    core::upgrade,
    dns::TokioDnsConfig,
    futures::StreamExt,
    gossipsub::{
        self, Gossipsub, GossipsubConfig, GossipsubEvent, IdentTopic, MessageAuthenticity,
    },
    identity,
    mdns::{Mdns, MdnsConfig, MdnsEvent},
    noise,
    swarm::{Swarm, SwarmBuilder, SwarmEvent},
    tcp::tokio::Transport as TcpTransport,
    yamux, Multiaddr, PeerId, Transport,
};
use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::error::Error;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tracing::{error, info};

pub static P2P_NODE: Lazy<Mutex<P2PNode>> = Lazy::new(|| Mutex::new(P2PNode::new()));

pub struct P2PNode {
    pub peer_id: PeerId,
    swarm: Swarm<gossipsub::Behaviour>,
    subscribed_topics: HashSet<String>,
}

impl P2PNode {
    pub fn new() -> Self {
        let id_keys = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(id_keys.public());

        let noise_keys = noise::Keypair::<noise::X25519Spec>::new()
            .into_authentic(&id_keys)
            .expect("Signing libp2p-noise static DH keypair failed.");

        let transport = TcpTransport::default()
            .upgrade(upgrade::Version::V1)
            .authenticate(noise::NoiseConfig::xx(noise_keys).into_authenticated())
            .multiplex(yamux::YamuxConfig::default())
            .boxed();

        let gossipsub_config = gossipsub::Config::default();
        let mut behaviour = gossipsub::Behaviour::new(
            MessageAuthenticity::Signed(id_keys.clone()),
            gossipsub_config,
        )
        .expect("Correct configuration");

        let swarm = Swarm::with_tokio_executor(transport, behaviour, peer_id);

        Self {
            peer_id,
            swarm,
            subscribed_topics: HashSet::new(),
        }
    }

    pub fn connect_to_peer(&mut self, addr: String) {
        let addr: Multiaddr = addr.parse().expect("Invalid multiaddr");
        Swarm::dial(&mut self.swarm, addr).expect("Dial failed");
    }

    pub fn publish_message(&mut self, topic: String, data: Vec<u8>) {
        let topic = IdentTopic::new(topic);
        let _ = self.swarm.behaviour_mut().publish(topic, data);
    }

    pub fn subscribe_topic(&mut self, topic: String) {
        if self.subscribed_topics.contains(&topic) {
            return;
        }
        let topic = IdentTopic::new(topic.clone());
        self.swarm.behaviour_mut().subscribe(&topic);
        self.subscribed_topics.insert(topic.hash().to_string());
    }

    pub fn get_peer_id(&self) -> String {
        self.peer_id.to_string()
    }

    /// Search currently connected peers.
    pub fn search_peers(&mut self) -> Result<Vec<String>, anyhow::Error> {
        let peers = self
            .swarm
            .behaviour()
            .all_peers()
            .map(|(peer_id, _)| peer_id.to_string())
            .collect();
        Ok(peers)
    }
}

pub fn start_node() {
    std::thread::spawn(|| {
        let runtime = Runtime::new().expect("Failed to create Tokio runtime");

        runtime.block_on(async {
            let id_keys = identity::Keypair::generate_ed25519();
            let peer_id = PeerId::from(id_keys.public());
            info!("Local peer id: {:?}", peer_id);

            let transport = TcpTransport::default()
                .upgrade(upgrade::Version::V1)
                .authenticate(noise::NoiseAuthenticated::xx(&id_keys).unwrap())
                .multiplex(yamux::YamuxConfig::default())
                .boxed();

            let gossipsub_config = GossipsubConfig::default();
            let mut gossipsub = Gossipsub::new(
                MessageAuthenticity::Signed(id_keys.clone()),
                gossipsub_config,
            )
            .expect("Correct Gossipsub config");

            let topic = IdentTopic::new("mycelium");
            gossipsub.subscribe(&topic).unwrap();

            let mdns = Mdns::new(MdnsConfig::default())
                .await
                .expect("Failed to create mDNS");

            let mut swarm = {
                let behaviour =
                    SwarmBuilder::with_tokio_executor((gossipsub, mdns), peer_id).build();
                behaviour
            };

            // 明示的にリッスンアドレスを追加
            swarm
                .listen_on("/ip4/0.0.0.0/tcp/4001".parse().unwrap())
                .expect("Failed to start listening");

            loop {
                match swarm.select_next_some().await {
                    SwarmEvent::Behaviour((
                        gossipsub::GossipsubEvent::Message { message, .. },
                        _,
                    )) => {
                        info!("Received: {:?}", String::from_utf8_lossy(&message.data));
                    }
                    SwarmEvent::Behaviour((
                        gossipsub::GossipsubEvent::Subscribed { peer_id, .. },
                        _,
                    )) => {
                        info!("Peer subscribed: {:?}", peer_id);
                    }
                    SwarmEvent::Behaviour((
                        gossipsub::GossipsubEvent::Unsubscribed { peer_id, .. },
                        _,
                    )) => {
                        info!("Peer unsubscribed: {:?}", peer_id);
                    }
                    SwarmEvent::Behaviour((MdnsEvent::Discovered(peers), _)) => {
                        for (peer_id, _addr) in peers {
                            info!("Discovered peer: {:?}", peer_id);
                        }
                    }
                    SwarmEvent::NewListenAddr { address, .. } => {
                        info!("Listening on {:?}", address);
                    }
                    _ => {}
                }
            }
        });
    });
}
