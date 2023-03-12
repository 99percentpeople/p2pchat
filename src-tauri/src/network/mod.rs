pub mod behaviour;
pub mod message;

use crate::error::NetworkError;
use crate::models::{GroupId, GroupInfo, GroupMessage};

/// The network module, encapsulating all network related logic.
use futures::StreamExt;

use libp2p::gossipsub::{GossipsubEvent, MessageId, Sha256Topic};
use libp2p::identity::ed25519;
use libp2p::request_response::{
    ProtocolSupport, RequestId, RequestResponse, RequestResponseEvent, RequestResponseMessage,
    ResponseChannel,
};
use libp2p::swarm::derive_prelude::ListenerId;
use libp2p::swarm::{keep_alive, Swarm, SwarmBuilder, SwarmEvent};
use libp2p::{gossipsub, mdns};
use libp2p::{identity, Multiaddr, PeerId};
use std::collections::hash_map::DefaultHasher;
use std::collections::{hash_map, HashMap};
use std::error::Error;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{mpsc, oneshot, Mutex};

use self::behaviour::*;
use self::message::*;

pub struct Network {
    pub client: Client,
    pub peer_id: PeerId,
    pub event_loop: EventLoop,
    pub event_receiver: mpsc::Receiver<InboundEvent>,
}

pub fn new(secret_key_seed: Option<u8>) -> anyhow::Result<Network> {
    // Create a public/private key pair, either random or based on a seed.
    let id_keys = match secret_key_seed {
        Some(seed) => {
            let mut bytes = [0u8; 32];
            bytes[0] = seed;
            let secret_key = ed25519::SecretKey::from_bytes(&mut bytes).expect(
                "this returns `Err` only if the length is wrong; the length is correct; qed",
            );
            identity::Keypair::Ed25519(secret_key.into())
        }
        None => identity::Keypair::generate_ed25519(),
    };
    let peer_id = id_keys.public().to_peer_id();
    // To content-address message, we can take the hash of message and use it as an ID.
    let message_id_fn = |message: &gossipsub::GossipsubMessage| {
        let mut s = DefaultHasher::new();
        message.data.hash(&mut s);
        gossipsub::MessageId::from(s.finish().to_string())
    };
    // Set a custom gossipsub configuration
    let gossipsub_config = gossipsub::GossipsubConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(10)) // This is set to aid debugging by not cluttering the log space
        .validation_mode(gossipsub::ValidationMode::Strict) // This sets the kind of message validation. The default is Strict (enforce message signing)
        .message_id_fn(message_id_fn) // content-address messages. No two messages of the same content will be propagated.
        .build()
        .expect("Valid config");

    // build a gossipsub network behaviour
    let gossipsub = gossipsub::Gossipsub::new(
        gossipsub::MessageAuthenticity::Signed(id_keys.clone()),
        gossipsub_config,
    )
    .expect("Correct configuration");

    // Create a Request-Response protocol supporting the FileExchange protocol.
    let request_response = RequestResponse::new(
        FileExchangeCodec(),
        std::iter::once((FileExchangeProtocol(), ProtocolSupport::Full)),
        Default::default(),
    );
    // Create a mdns behaviour
    let mdns = mdns::tokio::Behaviour::new(mdns::Config::default()).unwrap();

    let behaviour = ComposedBehaviour {
        mdns,
        request_response,
        gossipsub,
        keep_alive: keep_alive::Behaviour::default(),
    };
    // Build the Swarm, connecting the lower layer transport logic with the
    // higher layer network behaviour logic.
    let swarm = SwarmBuilder::with_tokio_executor(
        libp2p::tokio_development_transport(id_keys)?,
        behaviour,
        peer_id,
    )
    .build();

    let (command_sender, command_receiver) = mpsc::channel(100);
    let (event_sender, event_receiver) = mpsc::channel::<InboundEvent>(100);

    let network = Network {
        client: Client {
            sender: command_sender,
            local_peer_id: peer_id,
            listeners: Arc::new(Mutex::new(HashMap::new())),
            pending_new_group: Arc::new(Mutex::new(None)),
        },
        peer_id,
        event_loop: EventLoop::new(swarm, command_receiver, event_sender),
        event_receiver,
    };

    Ok(network)
}

#[derive(Debug, Clone)]
pub struct Client {
    sender: mpsc::Sender<Command>,
    local_peer_id: PeerId,
    pub listeners: Arc<Mutex<HashMap<ListenerId, Vec<Multiaddr>>>>,
    pub pending_new_group: Arc<Mutex<Option<(GroupId, GroupInfo)>>>,
}

impl Client {
    /// Listen for incoming connections on the given address.
    pub async fn start_listening(&self, addr: Multiaddr) -> Result<ListenerId, NetworkError> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::StartListen {
                addr: addr.clone(),
                sender,
            })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }
    /// Stop listening.
    pub async fn stop_listening(&self, listeners: Vec<ListenerId>) -> Result<(), NetworkError> {
        let (sender, receiver) = oneshot::channel();

        self.sender
            .send(Command::StopListen { sender, listeners })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }
    /// Dial the given peer at the given address.
    pub async fn dial(&self, peer_id: PeerId, addr: Multiaddr) -> Result<(), NetworkError> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::Dial {
                peer_id,
                addr,
                sender,
            })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }

    /// Request the content of the given file from the given peer.
    pub async fn request(&self, peer: PeerId, request: Request) -> Result<Response, NetworkError> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::Request {
                peer,
                request,
                sender,
            })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not be dropped.")
    }

    /// Respond with the provided file content to the given request.
    pub async fn response(&self, response: Response, channel: ResponseChannel<FileResponse>) {
        self.sender
            .send(Command::Response { response, channel })
            .await
            .expect("Command receiver not to be dropped.");
    }

    pub async fn publish(
        &self,
        topic: Sha256Topic,
        message: Message,
    ) -> Result<MessageId, NetworkError> {
        let (sender, receiver) = oneshot::channel();
        let _ = self
            .sender
            .send(Command::Publish {
                topic,
                message,
                sender,
            })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }

    pub async fn subscribe(&self, topic: Sha256Topic) -> Result<(), NetworkError> {
        let (sender, receiver) = oneshot::channel();
        let _ = self
            .sender
            .send(Command::Subscribe { topic, sender })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }

    pub async fn unsubscribe(&self, topic: Sha256Topic) -> Result<(), NetworkError> {
        let (sender, receiver) = oneshot::channel();
        let _ = self
            .sender
            .send(Command::Unsubscribe { topic, sender })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }
    pub fn local_peer_id(&self) -> PeerId {
        self.local_peer_id.clone()
    }
    pub async fn connected_peers(&self) -> Vec<PeerId> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::ConnectedPeers { sender })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }
    pub async fn new_group(
        &self,
        group_id: GroupId,
        group_info: GroupInfo,
    ) -> Result<(), NetworkError> {
        *self.pending_new_group.lock().await = Some((group_id.clone(), group_info));
        self.subscribe(group_id.topic()).await?;
        Ok(())
    }
}

pub struct EventLoop {
    swarm: Swarm<ComposedBehaviour>,
    command_receiver: mpsc::Receiver<Command>,
    event_sender: mpsc::Sender<InboundEvent>,
    pending_dial: HashMap<PeerId, oneshot::Sender<Result<(), NetworkError>>>,
    pending_request_file: HashMap<RequestId, oneshot::Sender<Result<Response, NetworkError>>>,
}

impl EventLoop {
    fn new(
        swarm: Swarm<ComposedBehaviour>,
        command_receiver: mpsc::Receiver<Command>,
        event_sender: mpsc::Sender<InboundEvent>,
    ) -> Self {
        Self {
            swarm,
            command_receiver,
            event_sender,
            pending_dial: Default::default(),
            pending_request_file: Default::default(),
        }
    }

    pub async fn run(mut self) {
        loop {
            tokio::select! {
                event = self.swarm.next() => self.handle_event(event.expect("Swarm stream to be infinite.")).await,
                command = self.command_receiver.recv() => match command {
                    Some(c) => self.handle_command(c).await,
                    // Command channel closed, thus shutting down the network event loop.
                    None =>  return,
                },
            }
        }
    }

    async fn handle_event<THandlerErr: Debug + Error>(
        &mut self,
        event: SwarmEvent<ComposedEvent, THandlerErr>,
    ) {
        match event {
            SwarmEvent::Behaviour(ComposedEvent::Gossipsub(event)) => match event {
                GossipsubEvent::Message {
                    propagation_source: _,
                    message_id,
                    message,
                } => {
                    let group_message =
                        serde_json::from_slice::<GroupMessage>(&message.data).unwrap();
                    let _ = self
                        .event_sender
                        .send(InboundEvent::Message {
                            message_id,
                            topic: message.topic,
                            message: group_message,
                        })
                        .await
                        .expect("Event receiver not to be dropped.");
                }
                GossipsubEvent::Subscribed { peer_id, topic } => {
                    log::info!("{:?} Subscribed to topic: {:?}", peer_id, topic);

                    let _ = self
                        .event_sender
                        .send(InboundEvent::Subscribed { peer_id, topic })
                        .await
                        .expect("Event receiver not to be dropped.");
                }
                GossipsubEvent::Unsubscribed { peer_id, topic } => {
                    let _ = self
                        .event_sender
                        .send(InboundEvent::Unsubscribed { peer_id, topic })
                        .await
                        .expect("Event receiver not to be dropped.");
                }
                GossipsubEvent::GossipsubNotSupported { .. } => {}
            },
            SwarmEvent::Behaviour(ComposedEvent::RequestResponse(
                RequestResponseEvent::Message { message, .. },
            )) => match message {
                RequestResponseMessage::Request {
                    request, channel, ..
                } => {
                    let _ = self
                        .event_sender
                        .send(InboundEvent::InboundRequest {
                            request: request.0,
                            channel: Arc::new(Mutex::new(Some(channel))),
                        })
                        .await
                        .expect("Event receiver not to be dropped.");
                }
                RequestResponseMessage::Response {
                    request_id,
                    response,
                } => {
                    let _ = self
                        .pending_request_file
                        .remove(&request_id)
                        .expect("Request to still be pending.")
                        .send(Ok(response.0));
                }
            },
            SwarmEvent::Behaviour(ComposedEvent::RequestResponse(
                RequestResponseEvent::OutboundFailure {
                    request_id, error, ..
                },
            )) => {
                let _ = self
                    .pending_request_file
                    .remove(&request_id)
                    .expect("Request to still be pending.")
                    .send(Err(error.into()));
            }
            SwarmEvent::Behaviour(ComposedEvent::RequestResponse(
                RequestResponseEvent::ResponseSent { .. },
            )) => {}
            SwarmEvent::Behaviour(ComposedEvent::Mdns(event)) => match event {
                mdns::Event::Discovered(list) => {
                    for (peer_id, addr) in list {
                        log::debug!("Discovered {:?} at {:?}", peer_id, addr);
                        self.swarm
                            .behaviour_mut()
                            .gossipsub
                            .add_explicit_peer(&peer_id);
                        self.event_sender
                            .send(InboundEvent::PeerDiscovered { peer_id })
                            .await
                            .unwrap();
                    }
                }
                mdns::Event::Expired(list) => {
                    for (peer_id, addr) in list {
                        log::debug!("Expired {:?} at {:?}", peer_id, addr);

                        self.swarm
                            .behaviour_mut()
                            .gossipsub
                            .remove_explicit_peer(&peer_id);
                        self.event_sender
                            .send(InboundEvent::PeerExpired { peer_id })
                            .await
                            .unwrap();
                    }
                }
            },
            SwarmEvent::NewListenAddr {
                address,
                listener_id,
            } => {
                self.event_sender
                    .send(InboundEvent::NewListenAddr {
                        listener_id,
                        address,
                    })
                    .await
                    .unwrap();
            }
            SwarmEvent::ListenerClosed {
                listener_id,
                addresses,
                reason,
            } => {
                self.event_sender
                    .send(InboundEvent::ListenerClosed {
                        listener_id,
                        addresses: addresses.clone(),
                    })
                    .await
                    .unwrap();

                match reason {
                    Ok(()) => addresses.iter().for_each(|address| {
                        log::info!(
                            "Local node is no longer listening on {:?} {:?}",
                            listener_id,
                            address
                        )
                    }),
                    Err(e) => addresses.iter().for_each(|address| {
                        log::warn!(
                            "Local node is no longer listening on {:?} {:?}: {:?}",
                            listener_id,
                            address,
                            e
                        )
                    }),
                }
            }
            SwarmEvent::IncomingConnection { .. } => {}
            SwarmEvent::ConnectionEstablished {
                peer_id, endpoint, ..
            } => {
                if endpoint.is_dialer() {
                    if let Some(sender) = self.pending_dial.remove(&peer_id) {
                        let _ = sender.send(Ok(()));
                    }
                }
            }
            SwarmEvent::ConnectionClosed { .. } => {}
            SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                if let Some(peer_id) = peer_id {
                    if let Some(sender) = self.pending_dial.remove(&peer_id) {
                        let _ = sender.send(Err(error.into()));
                    }
                }
            }
            SwarmEvent::IncomingConnectionError { .. } => {}
            SwarmEvent::Dialing(..) => {}
            e => log::debug!("{e:?}"),
        }
    }

    async fn handle_command(&mut self, command: Command) {
        match command {
            Command::StartListen { addr, sender } => {
                let _ = match self.swarm.listen_on(addr) {
                    Ok(listener_id) => sender.send(Ok(listener_id)),
                    Err(e) => sender.send(Err(e.into())),
                };
            }
            Command::StopListen { sender, listeners } => {
                for listener_id in listeners {
                    if !self.swarm.remove_listener(listener_id) {
                        log::warn!("Listener {:?} not found.", listener_id);
                    };
                }
                let _ = sender.send(Ok(()));
            }
            Command::Dial {
                peer_id,
                addr,
                sender,
            } => {
                if let hash_map::Entry::Vacant(e) = self.pending_dial.entry(peer_id) {
                    match self.swarm.dial(addr) {
                        Ok(()) => {
                            e.insert(sender);
                        }
                        Err(e) => {
                            let _ = sender.send(Err(e.into()));
                        }
                    }
                } else {
                    log::warn!("Already dialing peer {peer_id}");
                }
            }
            Command::Request {
                peer,
                request,
                sender,
            } => {
                let request_id = self
                    .swarm
                    .behaviour_mut()
                    .request_response
                    .send_request(&peer, FileRequest(request));
                self.pending_request_file.insert(request_id, sender);
            }
            Command::Response { response, channel } => {
                self.swarm
                    .behaviour_mut()
                    .request_response
                    .send_response(channel, FileResponse(response))
                    .expect("Connection to peer to be still open.");
            }
            Command::Publish {
                topic,
                message,
                sender,
            } => {
                let group_message =
                    GroupMessage::new(message, self.swarm.local_peer_id().to_owned());
                let res = self
                    .swarm
                    .behaviour_mut()
                    .gossipsub
                    .publish(topic.clone(), serde_json::to_vec(&group_message).unwrap());

                let res = match res {
                    Ok(message_id) => {
                        self.event_sender
                            .send(InboundEvent::Message {
                                message_id: message_id.clone(),
                                topic: topic.hash(),
                                message: group_message,
                            })
                            .await
                            .unwrap();
                        Ok(message_id)
                    }
                    Err(e) => Err(e.into()),
                };

                sender.send(res).expect("Receiver not to be dropped");
            }
            Command::Subscribe { topic, sender } => {
                match self.swarm.behaviour_mut().gossipsub.subscribe(&topic) {
                    Ok(res) => {
                        if !res {
                            log::warn!("Already subscribed to topic {:?}", topic);
                        }
                        let local_peer_id = self.swarm.local_peer_id().clone();
                        self.event_sender
                            .send(InboundEvent::Subscribed {
                                peer_id: local_peer_id,
                                topic: topic.hash(),
                            })
                            .await
                            .unwrap();
                        let _ = sender.send(Ok(()));
                    }
                    Err(e) => {
                        let _ = sender.send(Err(e.into()));
                    }
                }
            }
            Command::Unsubscribe { topic, sender } => {
                match self.swarm.behaviour_mut().gossipsub.unsubscribe(&topic) {
                    Ok(res) => {
                        if !res {
                            log::warn!("Already unsubscribed from topic {:?}", topic);
                        }
                        let local_peer_id = self.swarm.local_peer_id().clone();
                        self.event_sender
                            .send(InboundEvent::Unsubscribed {
                                peer_id: local_peer_id,
                                topic: topic.hash(),
                            })
                            .await
                            .unwrap();
                        let _ = sender.send(Ok(()));
                    }
                    Err(e) => {
                        let _ = sender.send(Err(e.into()));
                    }
                }
            }
            Command::ConnectedPeers { sender } => {
                let peers = self.swarm.connected_peers().cloned().collect();
                let _ = sender.send(peers);
            }
        }
    }
}

#[derive(Debug)]
enum Command {
    StartListen {
        addr: Multiaddr,
        sender: oneshot::Sender<Result<ListenerId, NetworkError>>,
    },
    StopListen {
        sender: oneshot::Sender<Result<(), NetworkError>>,
        listeners: Vec<ListenerId>,
    },
    Dial {
        peer_id: PeerId,
        addr: Multiaddr,
        sender: oneshot::Sender<Result<(), NetworkError>>,
    },
    Request {
        peer: PeerId,
        request: Request,
        sender: oneshot::Sender<Result<Response, NetworkError>>,
    },
    Response {
        response: Response,
        channel: ResponseChannel<FileResponse>,
    },
    Publish {
        topic: Sha256Topic,
        message: Message,
        sender: oneshot::Sender<Result<MessageId, NetworkError>>,
    },
    Subscribe {
        topic: Sha256Topic,
        sender: oneshot::Sender<Result<(), NetworkError>>,
    },
    Unsubscribe {
        topic: Sha256Topic,
        sender: oneshot::Sender<Result<(), NetworkError>>,
    },
    ConnectedPeers {
        sender: oneshot::Sender<Vec<PeerId>>,
    },
}
