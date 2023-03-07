use crate::{
    error::{ManagerError, NetworkError},
    function::HandleInboundEvent,
    models::{GroupId, GroupInfo, GroupMessage, GroupState, UserInfo, UserState},
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
use tokio::sync::Mutex;

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
}

#[async_trait]
impl HandleInboundEvent for UserManager {
    async fn handle_event(
        &mut self,
        event: InboundEvent,
        mut client: Client,
    ) -> Result<(), NetworkError> {
        match event {
            InboundEvent::InboundRequest { request, channel } => {
                if let Request::User(peer_id) = request {
                    if let Some(user_info) = self.get_user_info(&peer_id).await {
                        if let Some(channel) = channel.lock().await.take() {
                            client.response(Response::User(user_info), channel).await;
                        }
                    }
                }
            }
            InboundEvent::PeerDiscovered { peer_id } => {
                if !self.has_user(&peer_id).await {
                    match client.request(peer_id, Request::User(peer_id)).await {
                        Ok(Response::User(user_info)) => {
                            self.add_user(peer_id, user_info).await;
                        }
                        Ok(_) => log::warn!("Unexpected response"),
                        Err(err) => {
                            Err(err)?;
                        }
                    }
                }
            }
            InboundEvent::Subscribed { peer_id, topic } => {
                if !self.has_user(&peer_id).await {
                    match client.request(peer_id, Request::User(peer_id)).await {
                        Ok(Response::User(user_info)) => {
                            self.add_user(peer_id, user_info).await;
                        }
                        Ok(_) => log::warn!("Unexpected response"),
                        Err(err) => {
                            Err(err)?;
                        }
                    }
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
