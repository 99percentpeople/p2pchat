use std::collections::HashMap;
use tauri::{AppHandle, Manager};

use crate::{
    error::NetworkError,
    models::{Group, GroupInfo, GroupMessage},
};
use libp2p::{self, Multiaddr, PeerId};
use tokio::sync::mpsc;

pub struct FrontendEventLoop {
    pub(super) app: AppHandle,
    pub(super) frontend_receiver: mpsc::Receiver<FrontendEvent>,
}
#[derive(Debug)]
pub enum FrontendEvent {
    Listen {
        listeners: Vec<Multiaddr>,
    },
    Message {
        group: Group,
        message: GroupMessage,
    },
    Subscribed {
        group: Group,
        peer_id: PeerId,
    },
    Unsubscribed {
        group: Group,
        peer_id: PeerId,
    },
    GroupUpdate {
        group_list: HashMap<Group, GroupInfo>,
    },
    BackendError(NetworkError),
}

impl FrontendEventLoop {
    pub async fn run(mut self) {
        while let Some(event) = self.frontend_receiver.recv().await {
            let app = self.app.clone();
            tokio::spawn(async move {
                match event {
                    FrontendEvent::Listen { listeners } => {
                        app.emit_all("listen", listeners).unwrap();
                    }
                    FrontendEvent::Message { group, message } => {
                        app.emit_all("message", (group, message)).unwrap();
                    }
                    FrontendEvent::BackendError(err) => {
                        log::error!("{err}");
                        app.emit_all("error", err.to_string()).unwrap()
                    }
                    FrontendEvent::Subscribed { group, peer_id } => {
                        app.emit_all("subscribed", (group, peer_id)).unwrap();
                    }
                    FrontendEvent::Unsubscribed { group, peer_id } => {
                        app.emit_all("unsubscribed", (group, peer_id)).unwrap();
                    }
                    FrontendEvent::GroupUpdate { group_list } => {
                        app.emit_all("group-update", group_list).unwrap();
                    }
                }
            });
        }
    }
}
