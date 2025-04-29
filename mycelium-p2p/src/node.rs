use crate::proto;
use async_trait::async_trait;
use futures::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
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
    time::Duration,
};
use tokio::{io, runtime};

pub struct MyNode {
    swarm: libp2p::Swarm<MyBehaviour>,
    dbg_topics: HashSet<String>,
}

#[derive(NetworkBehaviour)]
struct MyBehaviour {
    gossipsub: gossipsub::Behaviour,
    mdns: mdns::tokio::Behaviour,
    request_response: request_response::Behaviour<ReactionCodec>,
}

#[derive(Clone)]
pub struct ReactionProtocol;

impl AsRef<str> for ReactionProtocol {
    fn as_ref(&self) -> &str {
        "/mycelium/reaction/1.0.0"
    }
}

#[derive(Clone)]
pub struct ReactionCodec;

#[async_trait]
impl request_response::Codec for ReactionCodec {
    type Protocol = ReactionProtocol;
    type Request = proto::SignedReaction;
    type Response = proto::SignedAck;

    async fn read_request<T>(
        &mut self,
        _protocol: &ReactionProtocol,
        io: &mut T,
    ) -> Result<Self::Request, std::io::Error>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        io.read_to_end(&mut buf).await?;
        proto::SignedReaction::decode(buf.as_slice())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    async fn read_response<T>(
        &mut self,
        _protocol: &ReactionProtocol,
        io: &mut T,
    ) -> Result<Self::Response, std::io::Error>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        io.read_to_end(&mut buf).await?;
        proto::SignedAck::decode(buf.as_slice())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    async fn write_request<T>(
        &mut self,
        _protocol: &ReactionProtocol,
        io: &mut T,
        req: Self::Request,
    ) -> Result<(), std::io::Error>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let mut buf = Vec::new();
        req.encode(&mut buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        io.write_all(&buf).await?;
        Ok(())
    }

    async fn write_response<T>(
        &mut self,
        _protocol: &ReactionProtocol,
        io: &mut T,
        res: Self::Response,
    ) -> Result<(), std::io::Error>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let mut buf = Vec::new();
        res.encode(&mut buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        io.write_all(&buf).await?;
        Ok(())
    }
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

                let request_response = request_response::Behaviour::with_codec(
                    ReactionCodec,
                    [(ReactionProtocol, request_response::ProtocolSupport::Full)],
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

    pub fn send_reaction(&mut self, peer: impl AsRef<str>, reaction_bytes: Vec<u8>) {
        let reaction = proto::SignedReaction::decode(reaction_bytes.as_slice())
            .expect("Failed to decode reaction");
        let peer_id = peer.as_ref().parse().expect("Invalid peer ID");
        self.swarm
            .behaviour_mut()
            .request_response
            .send_request(&peer_id, reaction);
    }

    #[expect(dead_code)]
    pub fn send_ack(
        &mut self,
        ch: request_response::ResponseChannel<proto::SignedAck>,
        ack: proto::SignedAck,
    ) -> Result<(), proto::SignedAck> {
        self.swarm
            .behaviour_mut()
            .request_response
            .send_response(ch, ack)
    }

    pub fn local_peer_id(&self) -> &PeerId {
        self.swarm.local_peer_id()
    }

    pub fn connected_peers(&self) -> impl Iterator<Item = &PeerId> {
        self.swarm.connected_peers()
    }

    pub fn discovered_nodes(&self) -> impl Iterator<Item = &PeerId> {
        self.swarm.behaviour().mdns.discovered_nodes()
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
                        SwarmEvent::Behaviour(MyBehaviourEvent::Gossipsub(event)) => match event {
                            gossipsub::Event::Message { message, .. } => println!("Received: {:?}", String::from_utf8_lossy(&message.data)),
                            gossipsub::Event::Subscribed { peer_id, .. } => println!("Peer subscribed: {:?}", peer_id),
                            gossipsub::Event::Unsubscribed { peer_id, .. } => println!("Peer unsubscribed: {:?}", peer_id),
                            _ => {}
                        }
                        SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(event)) => match event {
                            mdns::Event::Discovered(peers) => {
                                for (peer_id, _addr) in peers {
                                    println!("Discovered peer: {:?}", peer_id);
                                }
                            }
                            mdns::Event::Expired(peers) => {
                                for (peer_id, _addr) in peers {
                                    println!("Expired peer: {:?}", peer_id);
                                }
                            }
                        }
                        SwarmEvent::Behaviour(MyBehaviourEvent::RequestResponse(event)) => {
                            match event {
                                request_response::Event::Message { message, .. } => {
                                    if let request_response::Message::Request { request, .. } = message {
                                        // request: SignedReaction
                                        // 1) verify 2) store 3) gen SignedAck 4) send_ack
                                        println!("Reaction received: {:?}", request);
                                    }
                                }
                                _ => {}
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
