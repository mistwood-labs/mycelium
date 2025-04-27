use futures::StreamExt;
use libp2p::{
    core::upgrade,
    dns::TokioDnsConfig,
    gossipsub::{self, GossipsubEvent, IdentTopic, MessageAuthenticity},
    identity, noise,
    swarm::{Swarm, SwarmEvent},
    tcp::TokioTcpConfig,
    yamux, Multiaddr, PeerId, Transport,
};
use libp2p::{
    gossipsub::{Gossipsub, GossipsubConfig, GossipsubEvent, IdentTopic, MessageAuthenticity},
    identity, noise,
    swarm::SwarmBuilder,
    tcp::TcpConfig,
    yamux, Multiaddr, PeerId, Swarm, Transport,
};
use std::collections::HashSet;
use std::error::Error;
use std::time::Duration;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tracing::{error, info};

pub struct P2PNode {
    pub peer_id: PeerId,
    swarm: Swarm<gossipsub::Behaviour>,
    subscribed_topics: HashSet<String>,
}

impl P2PNode {
    pub fn new(port: u16) -> Self {
        let id_keys = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(id_keys.public());

        let noise_keys = noise::Keypair::<noise::X25519Spec>::new()
            .into_authentic(&id_keys)
            .expect("Signing libp2p-noise static DH keypair failed.");

        let transport = TokioTcpConfig::new()
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
        self.subscribed_topics.insert(topic.id().to_string());
    }

    pub fn get_peer_id(&self) -> String {
        self.peer_id.to_string()
    }
}

pub fn start_node() {
    std::thread::spawn(|| {
        // トーキオランタイムを生成
        let runtime = Runtime::new().expect("Failed to create Tokio runtime");

        runtime.block_on(async {
            // 自己ID作成
            let id_keys = identity::Keypair::generate_ed25519();
            let peer_id = PeerId::from(id_keys.public());
            info!("Local peer id: {:?}", peer_id);

            // トランスポート設定
            let transport = TcpConfig::new()
                .upgrade(upgrade::Version::V1)
                .authenticate(noise::NoiseAuthenticated::xx(&id_keys).unwrap())
                .multiplex(yamux::YamuxConfig::default())
                .boxed();

            // Gossipsub設定
            let gossipsub_config = GossipsubConfig::default();
            let mut gossipsub = Gossipsub::new(
                MessageAuthenticity::Signed(id_keys.clone()),
                gossipsub_config,
            )
            .expect("Correct Gossipsub config");

            // トピック登録
            let topic = IdentTopic::new("mycelium");
            gossipsub.subscribe(&topic).unwrap();

            // Swarm構築
            let mut swarm =
                SwarmBuilder::with_tokio_executor(transport, gossipsub, peer_id).build();

            // 簡易イベントループ
            loop {
                match swarm.select_next_some().await {
                    gossipsub::GossipsubEvent::Message { message, .. } => {
                        info!("Received: {:?}", String::from_utf8_lossy(&message.data));
                    }
                    gossipsub::GossipsubEvent::Subscribed { peer_id, .. } => {
                        info!("Peer subscribed: {:?}", peer_id);
                    }
                    gossipsub::GossipsubEvent::Unsubscribed { peer_id, .. } => {
                        info!("Peer unsubscribed: {:?}", peer_id);
                    }
                    _ => {}
                }
            }
        });
    });
}

use crate::node::SWARM;
use libp2p::gossipsub::IdentTopic;
use std::sync::MutexGuard;

/// Search currently connected peers.
pub fn search_peers() -> Result<Vec<String>, anyhow::Error> {
    let swarm = SWARM.lock().unwrap();
    let peers = swarm
        .behaviour()
        .gossipsub()
        .all_peers()
        .map(|(peer_id, _)| peer_id.to_string())
        .collect();
    Ok(peers)
}
