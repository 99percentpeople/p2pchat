use libp2p::{
    gossipsub::{GossipsubMessage, MessageId, TopicHash},
    request_response::ResponseChannel,
    swarm::derive_prelude::ListenerId,
    Multiaddr, PeerId,
};
use serde::{Deserialize, Serialize};

use crate::models::{FileInfo, Group, GroupInfo};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Message {
    Text(String),
    File(FileInfo),
}

#[derive(Debug)]
pub enum Event {
    InboundRequest {
        request: Request,
        channel: ResponseChannel<FileResponse>,
    },
    InboundMessage {
        propagation_source: PeerId,
        message_id: MessageId,
        message: GossipsubMessage,
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
    NewListenAddr {
        address: Multiaddr,
        listener_id: ListenerId,
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
}

#[derive(Debug, Clone)]
pub enum Response {
    File(Vec<u8>),
    Group((Group, GroupInfo)),
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
