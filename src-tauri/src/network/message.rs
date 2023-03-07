use libp2p::{
    gossipsub::{GossipsubMessage, MessageId, TopicHash},
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
    MessageReceived {
        message_id: MessageId,
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
        address: Multiaddr,
        listener_id: ListenerId,
    },
    ListenerClosed {
        listener_id: ListenerId,
        addresses: Vec<Multiaddr>,
    },
    PublishMessage {
        message: GroupMessage,
    },
    NewGroup {
        group_id: GroupId,
        group_info: GroupInfo,
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
// #[derive(Debug, Clone, PartialEq, Eq, Error)]
// pub enum ResponseError {
//     #[error("File not found: {0}")]
//     NotFoundError(String),
// }
// impl From<io::Error> for ResponseError {
//     fn from(value: io::Error) -> Self {
//         match value.kind() {
//             io::ErrorKind::NotFound => ResponseError::NotFoundError(value.to_string()),
//             _ => panic!("Unexpected error: {}", value),
//         }
//     }
// }

#[derive(Debug, Clone)]
pub struct FileResponse(pub Response);
