use crate::{network::protocol, proto};
use libp2p::{
    futures::StreamExt,
    gossipsub, mdns, noise, request_response,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId,
};
use prost::Message;
use std::{
    collections::{hash_map::DefaultHasher, HashSet},
    error::Error,
    hash::{Hash, Hasher},
    sync::Arc,
    time::Duration,
};
use tokio::sync::Mutex as TokioMutex;

pub struct MyNode {
    swarm: Arc<TokioMutex<libp2p::Swarm<MyBehaviour>>>,
    dbg_topics: HashSet<String>,
}

#[derive(NetworkBehaviour)]
struct MyBehaviour {
    gossipsub: gossipsub::Behaviour,
    mdns: mdns::tokio::Behaviour,
    request_response: request_response::Behaviour<protocol::ReactionCodec>,
}

impl MyNode {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
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
                    .build()?;
                // .map_err(io::Error::other)?; // Temporary hack because `build` does not return a proper `std::error::Error`.

                // build a gossipsub network behaviour
                let gossipsub = gossipsub::Behaviour::new(
                    gossipsub::MessageAuthenticity::Signed(key.clone()),
                    gossipsub_config,
                )?;

                let mdns = mdns::tokio::Behaviour::new(
                    mdns::Config::default(),
                    key.public().to_peer_id(),
                )?;

                let request_response = request_response::Behaviour::with_codec(
                    protocol::ReactionCodec,
                    [(
                        protocol::ReactionProtocol,
                        request_response::ProtocolSupport::Full,
                    )],
                    request_response::Config::default(),
                );

                Ok(MyBehaviour {
                    gossipsub,
                    mdns,
                    request_response,
                })
            })?
            .build();

        Ok(Self {
            swarm: Arc::new(TokioMutex::new(swarm)),
            dbg_topics: HashSet::new(),
        })
    }

    pub async fn connect_to_peer(&mut self, addr: &str) -> Result<(), Box<dyn Error>> {
        let addr: Multiaddr = addr.parse()?;
        self.swarm.lock().await.dial(addr)?;
        Ok(())
    }

    pub async fn publish_post(
        &mut self,
        topic: &str,
        post: proto::SignedPost,
    ) -> Result<(), Box<dyn Error>> {
        let topic = gossipsub::IdentTopic::new(topic);
        let mut post_bytes = Vec::new();
        post.encode(&mut post_bytes)?;
        self.swarm
            .lock()
            .await
            .behaviour_mut()
            .gossipsub
            .publish(topic, post_bytes)?;
        Ok(())
    }

    pub async fn subscribe_topic(&mut self, topic: &str) -> Result<(), Box<dyn Error>> {
        if self.dbg_topics.contains(topic) {
            return Err("Already subscribed to this topic".into());
        }
        let topic = gossipsub::IdentTopic::new(topic);
        self.swarm
            .lock()
            .await
            .behaviour_mut()
            .gossipsub
            .subscribe(&topic)?;
        self.dbg_topics.insert(topic.to_string());
        Ok(())
    }

    pub async fn send_reaction(
        &mut self,
        peer: &str,
        reaction: proto::SignedReaction,
    ) -> Result<(), Box<dyn Error>> {
        let peer_id = peer.parse()?;
        self.swarm
            .lock()
            .await
            .behaviour_mut()
            .request_response
            .send_request(&peer_id, reaction);
        Ok(())
    }

    pub async fn send_ack(
        &mut self,
        ch: request_response::ResponseChannel<proto::SignedAck>,
        ack: proto::SignedAck,
    ) -> Result<(), proto::SignedAck> {
        self.swarm
            .lock()
            .await
            .behaviour_mut()
            .request_response
            .send_response(ch, ack)
    }

    pub async fn local_peer_id(&self) -> PeerId {
        self.swarm.lock().await.local_peer_id().clone()
    }

    pub async fn connected_peers(&self) -> Vec<PeerId> {
        self.swarm.lock().await.connected_peers().cloned().collect()
    }

    pub async fn discovered_nodes(&self) -> Vec<PeerId> {
        self.swarm
            .lock()
            .await
            .behaviour()
            .mdns
            .discovered_nodes()
            .cloned()
            .collect()
    }

    pub async fn start(&mut self, addr: &str) -> Result<(), Box<dyn Error>> {
        let addr: Multiaddr = addr.parse()?;
        self.swarm.lock().await.listen_on(addr.clone())?;
        log::debug!("MyNode listening on {}", addr);

        let swarm_handle = Arc::clone(&self.swarm);
        tokio::spawn(async move {
            let mut swarm = swarm_handle.lock().await;
            loop {
                match swarm.select_next_some().await {
                    SwarmEvent::Behaviour(MyBehaviourEvent::Gossipsub(evt)) => {
                        log::debug!("Gossipsub event: {:?}", evt);
                    }
                    SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(evt)) => {
                        log::debug!("mDNS event: {:?}", evt);
                    }
                    SwarmEvent::Behaviour(MyBehaviourEvent::RequestResponse(evt)) => {
                        log::debug!("RequestResponse event: {:?}", evt);
                    }
                    SwarmEvent::NewListenAddr { address, .. } => {
                        log::debug!("New listen addr: {:?}", address);
                    }
                    other => {
                        log::trace!("Other event: {:?}", other);
                    }
                }
            }
        });

        // 3) return immediately so the FFI call completes
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<(), Box<dyn Error>> {
        // TODO
        Ok(())
    }
}
