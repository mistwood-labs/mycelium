use libp2p::{
    futures::StreamExt,
    gossipsub, mdns, noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId,
};
use once_cell::sync::Lazy;
use std::{
    collections::{hash_map::DefaultHasher, HashSet},
    error::Error,
    hash::{Hash, Hasher},
    sync::Mutex,
    time::Duration,
};
use tokio::{io, runtime};

pub static MY_NODE: Lazy<Mutex<MyNode>> =
    Lazy::new(|| Mutex::new(MyNode::new().expect("Failed to create node")));

pub struct MyNode {
    swarm: libp2p::Swarm<MyBehaviour>,
    dbg_topics: HashSet<String>,
}

#[derive(NetworkBehaviour)]
struct MyBehaviour {
    gossipsub: gossipsub::Behaviour,
    mdns: mdns::tokio::Behaviour,
}

impl MyNode {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let swarm = libp2p::SwarmBuilder::with_new_identity()
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_behaviour(|key| {
                // To content-address message, we can take the hash of message and use it as an ID.
                let message_id_fn = |message: &gossipsub::Message| {
                    let mut s = DefaultHasher::new();
                    message.data.hash(&mut s);
                    gossipsub::MessageId::from(s.finish().to_string())
                };

                // Set a custom gossipsub configuration
                let gossipsub_config = gossipsub::ConfigBuilder::default()
                    .heartbeat_interval(Duration::from_secs(10)) // This is set to aid debugging by not cluttering the log space
                    .validation_mode(gossipsub::ValidationMode::Strict) // This sets the kind of message validation. The default is Strict (enforce message
                    // signing)
                    .message_id_fn(message_id_fn) // content-address messages. No two messages of the same content will be propagated.
                    .build()
                    .map_err(io::Error::other)?; // Temporary hack because `build` does not return a proper `std::error::Error`.

                // build a gossipsub network behaviour
                let gossipsub = gossipsub::Behaviour::new(
                    gossipsub::MessageAuthenticity::Signed(key.clone()),
                    gossipsub_config,
                )?;

                let mdns = mdns::tokio::Behaviour::new(
                    mdns::Config::default(),
                    key.public().to_peer_id(),
                )?;

                Ok(MyBehaviour { gossipsub, mdns })
            })?
            .build();

        Ok(Self {
            swarm,
            dbg_topics: HashSet::new(),
        })
    }

    pub fn connect_to_peer(&mut self, addr: impl AsRef<str>) {
        let addr: Multiaddr = addr.as_ref().parse().expect("Invalid multiaddr");
        self.swarm.dial(addr).expect("Dial failed");
    }

    pub fn publish_message(&mut self, topic: impl Into<String>, data: Vec<u8>) {
        let topic = gossipsub::IdentTopic::new(topic);
        if let Err(e) = self.swarm.behaviour_mut().gossipsub.publish(topic, data) {
            eprintln!("Failed to publish message: {e:?}");
        }
    }

    pub fn subscribe_topic(&mut self, topic: String) {
        if self.dbg_topics.contains(&topic) {
            return;
        }
        let topic = gossipsub::IdentTopic::new(topic);
        if let Err(e) = self.swarm.behaviour_mut().gossipsub.subscribe(&topic) {
            eprintln!("Failed to subscribe to topic: {e:?}");
            return;
        }
        self.dbg_topics.insert(topic.to_string());
    }

    pub fn local_peer_id(&self) -> &PeerId {
        self.swarm.local_peer_id()
    }

    pub fn discovered_nodes(&self) -> impl Iterator<Item = &PeerId> {
        self.swarm.behaviour().mdns.discovered_nodes()
    }

    pub fn connected_peers(&self) -> impl Iterator<Item = &PeerId> {
        self.swarm.connected_peers()
    }

    pub fn start(&mut self, addr: impl AsRef<str>) {
        let addr: Multiaddr = addr.as_ref().parse().expect("Invalid multiaddr");
        let runtime = runtime::Runtime::new().expect("Failed to create runtime");

        runtime.block_on(async {
            self.swarm
                .listen_on(addr)
                .expect("Failed to start listening");

            loop {
                tokio::select! {
                    event = self.swarm.select_next_some() => match event {
                        SwarmEvent::Behaviour(MyBehaviourEvent::Gossipsub(
                            gossipsub::Event::Message { message, .. },
                        )) => {
                            println!("Received: {:?}", String::from_utf8_lossy(&message.data));
                        }
                        SwarmEvent::Behaviour(MyBehaviourEvent::Gossipsub(
                            gossipsub::Event::Subscribed { peer_id, .. },
                        )) => {
                            println!("Peer subscribed: {:?}", peer_id);
                        }
                        SwarmEvent::Behaviour(MyBehaviourEvent::Gossipsub(
                            gossipsub::Event::Unsubscribed { peer_id, .. },
                        )) => {
                            println!("Peer unsubscribed: {:?}", peer_id);
                        }
                        SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Discovered(peers))) => {
                            for (peer_id, _addr) in peers {
                                println!("Discovered peer: {:?}", peer_id);
                            }
                        }
                        SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Expired(peers))) => {
                            for (peer_id, _addr) in peers {
                                println!("Expired peer: {:?}", peer_id);
                            }
                        }
                        SwarmEvent::NewListenAddr { address, .. } => {
                            println!("Listening on {:?}", address);
                        }
                        _ => {}
                    }
                }
            }
        });
    }
}
