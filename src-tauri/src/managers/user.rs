use super::{AppManager, HandleInboundEvent, Invoke};
use crate::{
    chat_app::{frontend_event::FrontendEvent, AppState},
    error::{ManagerError, NetworkError},
    models::{UserInfo, UserState},
    network::{
        message::{InboundEvent, Request, Response},
        Client,
    },
};
use async_trait::async_trait;
use libp2p::{gossipsub::TopicHash, PeerId};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use tokio::sync::{mpsc, Mutex};

#[derive(Debug, Clone)]
pub struct UserManager {
    users: Arc<Mutex<HashMap<PeerId, UserInfo>>>,
    user_subscribe: Arc<Mutex<HashMap<PeerId, HashSet<TopicHash>>>>,
}

impl UserManager {
    pub fn new() -> Self {
        Self {
            users: Arc::new(Mutex::new(HashMap::new())),
            user_subscribe: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    pub async fn add_user(&self, peer_id: PeerId, user_info: UserInfo) {
        self.users.lock().await.insert(peer_id, user_info);
    }
    pub async fn remove_user(&self, peer_id: &PeerId) {
        self.users.lock().await.remove(peer_id);
    }
    async fn add_subscribe(&self, peer_id: PeerId, topic: TopicHash) -> Result<(), ManagerError> {
        if !self.users.lock().await.contains_key(&peer_id) {
            return Err(ManagerError::PeerNotExist(peer_id));
        }
        self.user_subscribe
            .lock()
            .await
            .entry(peer_id)
            .or_default()
            .insert(topic);
        Ok(())
    }
    async fn remove_subscribe(&self, peer_id: &PeerId, topic: &TopicHash) -> bool {
        if let Some(groups) = self.user_subscribe.lock().await.get_mut(peer_id) {
            groups.remove(topic)
        } else {
            false
        }
    }
    pub async fn get_user_info(&self, peer_id: &PeerId) -> Option<UserInfo> {
        self.users.lock().await.get(peer_id).cloned()
    }
    pub async fn change_user_status(&self, peer_id: &PeerId, status: UserState) {
        if let Some(user_info) = self.users.lock().await.get_mut(peer_id) {
            user_info.status = status;
        }
    }
    pub async fn get_user_subscribe(&self, peer_id: &PeerId) -> Option<Vec<TopicHash>> {
        self.user_subscribe
            .lock()
            .await
            .get(peer_id)
            .map(|groups| groups.iter().cloned().collect())
    }
    pub async fn has_user(&self, peer_id: &PeerId) -> bool {
        self.users.lock().await.contains_key(peer_id)
    }
    pub async fn get_users(&self) -> HashMap<PeerId, UserInfo> {
        self.users.lock().await.clone()
    }
}

#[async_trait]
impl HandleInboundEvent for UserManager {
    async fn handle_event(
        &mut self,
        event: InboundEvent,
        client: Client,
        state: AppState,
        sender: mpsc::Sender<FrontendEvent>,
    ) -> Result<(), NetworkError> {
        match event {
            InboundEvent::InboundRequest { request, channel } => {
                if let Request::User(peer_id) = request {
                    if let Some(user_info) = self.get_user_info(&peer_id).await {
                        if let Some(channel) = channel.lock().await.take() {
                            client.response(Response::User(user_info), channel).await;
                        }
                    } else if peer_id == client.local_peer_id() {
                        if let Some(channel) = channel.lock().await.take() {
                            client
                                .response(
                                    Response::User(state.local_user.lock().await.clone().into()),
                                    channel,
                                )
                                .await;
                        }
                    }
                }
            }
            InboundEvent::PeerDiscovered { peer_id } => {
                if !self.has_user(&peer_id).await {
                    match client.request(peer_id, Request::User(peer_id)).await {
                        Ok(Response::User(user_info)) => {
                            self.add_user(peer_id, user_info.clone()).await;
                            sender
                                .send(FrontendEvent::UserUpdate { peer_id, user_info })
                                .await
                                .unwrap();
                        }
                        Ok(_) => log::warn!("Unexpected response"),
                        Err(err) => {
                            Err(err)?;
                        }
                    }
                }
            }
            InboundEvent::PeerExpired { peer_id } => {
                if self.has_user(&peer_id).await {
                    self.change_user_status(&peer_id, UserState::Offline).await;
                    sender
                        .send(FrontendEvent::UserUpdate {
                            peer_id,
                            user_info: self.get_user_info(&peer_id).await.unwrap(),
                        })
                        .await
                        .unwrap();
                }
            }
            InboundEvent::Subscribed { peer_id, topic } => {
                if !self.has_user(&peer_id).await {
                    let (peer_id, user_info) = if peer_id != client.local_peer_id() {
                        match client.request(peer_id, Request::User(peer_id)).await {
                            Ok(Response::User(user_info)) => (peer_id, user_info),
                            _ => return Err(anyhow::anyhow!("user not found").into()),
                        }
                    } else {
                        (peer_id, state.local_user.lock().await.clone().into())
                    };
                    sender
                        .send(FrontendEvent::UserUpdate {
                            peer_id: peer_id.clone(),
                            user_info: user_info.clone(),
                        })
                        .await
                        .unwrap();
                    self.add_user(peer_id, user_info).await;
                }
                self.add_subscribe(peer_id, topic).await?;
            }
            InboundEvent::Unsubscribed { peer_id, topic } => {
                self.remove_subscribe(&peer_id, &topic).await;
            }
            _ => {}
        }

        Ok(())
    }
}

#[async_trait]
impl Invoke for UserManager {
    async fn invoke(
        &self,
        command: &str,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, ManagerError> {
        let value = match command {
            "get_user_info" if params.is_some() => {
                let peer_id = serde_json::from_value::<PeerId>(params.unwrap())?;
                serde_json::to_value(
                    self.get_user_info(&peer_id)
                        .await
                        .ok_or(ManagerError::PeerNotExist(peer_id))?,
                )?
            }
            "get_users" => serde_json::to_value(self.get_users().await)?,
            c => return Err(ManagerError::InvalidAction(c.to_string())),
        };
        Ok(value)
    }
}

impl AppManager for UserManager {
    fn name(&self) -> &'static str {
        "user"
    }
}
