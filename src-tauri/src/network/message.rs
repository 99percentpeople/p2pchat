use libp2p::{
    gossipsub::{MessageId, TopicHash},
    request_response::ResponseChannel,
    swarm::derive_prelude::ListenerId,
    Multiaddr, PeerId,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::models::{FileInfo, GroupId, GroupInfo, GroupMessage, UserInfo};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Message {
    Text(String),
    File(FileInfo),
}

#[derive(Debug, Clone)]
pub enum InboundEvent {
    InboundRequest {
        request: Request,
        channel: Arc<Mutex<Option<ResponseChannel<FileResponse>>>>,
    },
    Message {
        topic: TopicHash,
        message: GroupMessage,
    },
    Subscribed {
        peer_id: PeerId,
        topic: TopicHash,
    },
    Unsubscribed {
        peer_id: PeerId,
        topic: TopicHash,
    },
    PeerDiscovered {
        peer_id: PeerId,
    },
    PeerExpired {
        peer_id: PeerId,
    },
    NewListenAddr {
        listener_id: ListenerId,
        address: Multiaddr,
    },
    ListenerClosed {
        listener_id: ListenerId,
        addresses: Vec<Multiaddr>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    File(FileInfo),
    Group(TopicHash),
    User(PeerId),
}

#[derive(Debug, Clone)]
pub enum Response {
    File(Vec<u8>),
    Group((GroupId, GroupInfo)),
    User(UserInfo),
}
#[derive(Debug, Clone)]
pub struct FileResponse(pub Response);
