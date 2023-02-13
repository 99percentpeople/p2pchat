use std::collections::HashMap;
use tauri::{AppHandle, Manager};

use crate::{
    error::NetworkError,
    models::{GroupId, GroupInfo, GroupMessage, GroupState, UserInfo},
};
use libp2p::{self, Multiaddr, PeerId};
use tokio::sync::mpsc;

use super::AppState;

pub struct FrontendEventLoop {
    pub(super) app: AppHandle,
    pub(super) frontend_receiver: mpsc::Receiver<FrontendEvent>,
    pub(super) state: AppState,
}
#[derive(Debug)]
pub enum FrontendEvent {
    Listen {
        listeners: Vec<Multiaddr>,
    },
    Message {
        group_id: GroupId,
        message: GroupMessage,
    },
    Subscribed {
        group: GroupId,
        peer_id: PeerId,
    },
    Unsubscribed {
        group: GroupId,
        peer_id: PeerId,
    },
    GroupUpdate {
        group_id: GroupId,
        group_info: GroupInfo,
    },
    GroupStateUpdate {
        group_id: GroupId,
        group_state: GroupState,
    },
    UserUpdate {
        peer_id: PeerId,
        user_info: UserInfo,
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
                    FrontendEvent::Message { group_id, message } => {
                        app.emit_all("message", (group_id, message)).unwrap();
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
                    FrontendEvent::GroupUpdate {
                        group_id,
                        group_info,
                    } => {
                        app.emit_all(&format!("group-update"), (group_id, group_info))
                            .unwrap();
                    }
                    FrontendEvent::UserUpdate { peer_id, user_info } => {
                        app.emit_all(&format!("user-update"), (peer_id, user_info))
                            .unwrap();
                    }
                    FrontendEvent::GroupStateUpdate {
                        group_id,
                        group_state,
                    } => {
                        app.emit_all(&format!("group-state-update"), (group_id, group_state))
                            .unwrap();
                    }
                }
            });
        }
    }
}
