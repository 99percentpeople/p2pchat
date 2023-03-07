use crate::{
    error::NetworkError,
    function::HandleInboundEvent,
    models::{GroupId, GroupInfo, GroupMessage, GroupState},
    network::{
        message::{InboundEvent, Request, Response},
        Client,
    },
};
use async_trait::async_trait;
use libp2p::{gossipsub::TopicHash, PeerId};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct GroupManager {
    group_state: Arc<Mutex<HashMap<GroupId, GroupState>>>,
    groups: Arc<Mutex<HashMap<GroupId, GroupInfo>>>,
}

impl GroupManager {
    pub fn new() -> Self {
        Self {
            group_state: Arc::new(Mutex::new(HashMap::new())),
            groups: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    pub async fn add_group(&self, group_id: GroupId, group_info: GroupInfo) {
        self.groups
            .lock()
            .await
            .insert(group_id.clone(), group_info);
        self.group_state
            .lock()
            .await
            .insert(group_id, GroupState::new());
    }
    pub async fn remove_group(&self, group_id: &GroupId) {
        self.groups.lock().await.remove(group_id);
        self.group_state.lock().await.remove(group_id);
    }
    pub async fn get_groups(&self) -> HashMap<GroupId, GroupInfo> {
        self.groups.lock().await.clone()
    }
    pub async fn add_message<G: AsRef<GroupId>>(&self, group_id: G, message: GroupMessage) {
        if let Some(group_status) = self.group_state.lock().await.get_mut(group_id.as_ref()) {
            group_status.history.push(message);
        }
    }
    pub async fn get_group_info(&self, group_id: &GroupId) -> Option<GroupInfo> {
        self.groups.lock().await.get(group_id).cloned()
    }
    pub async fn get_group_status(&self, group_id: &GroupId) -> Option<GroupState> {
        self.group_state.lock().await.get(group_id).cloned()
    }
    pub async fn has_group(&self, group_id: &GroupId) -> bool {
        self.groups.lock().await.contains_key(group_id.as_ref())
    }
    pub async fn has_group_by_hash(&self, topic_hash: &TopicHash) -> bool {
        self.groups
            .lock()
            .await
            .keys()
            .any(|group_info| &group_info.topic().hash() == topic_hash)
    }
    pub async fn get_group_by_hash(&self, topic_hash: &TopicHash) -> Option<GroupId> {
        self.groups
            .lock()
            .await
            .keys()
            .find(|group_info| &group_info.topic().hash() == topic_hash)
            .cloned()
    }
    pub async fn is_group_exist(&self, group_id: &GroupId) -> bool {
        self.groups.lock().await.contains_key(group_id)
    }
    pub async fn add_subscribe(&self, group_id: &GroupId, peer_id: PeerId) {
        if let Some(group_status) = self.group_state.lock().await.get_mut(group_id) {
            group_status.subscribers.insert(peer_id);
        }
    }
    pub async fn remove_subscribe(&self, group_id: &GroupId, peer_id: &PeerId) {
        if let Some(group_status) = self.group_state.lock().await.get_mut(group_id) {
            group_status.subscribers.remove(peer_id);
        }
    }
}

#[async_trait]
impl HandleInboundEvent for GroupManager {
    async fn handle_event(
        &mut self,
        event: InboundEvent,
        mut client: Client,
    ) -> Result<(), NetworkError> {
        match event {
            InboundEvent::InboundRequest { request, channel } => match request {
                Request::Group(topic_hash) => {
                    if let Some(group) = self.get_group_by_hash(&topic_hash).await {
                        let info = self.get_group_info(&group).await.unwrap();
                        if let Some(channel) = channel.lock().await.take() {
                            client
                                .response(Response::Group((group, info)), channel)
                                .await;
                        }
                    } else {
                        log::warn!("group not found {topic_hash:?}");
                    }
                }
                _ => {}
            },
            InboundEvent::Subscribed { peer_id, topic } => {
                // if local peer is the one who create the group, then add the group to local
                match client.pending_new_group.take() {
                    Some((group_id, group_info)) if peer_id == client.local_peer_id() => {
                        self.add_group(group_id.clone(), group_info).await;
                        self.add_subscribe(&group_id, peer_id).await;
                        return Ok(());
                    }
                    _ => {}
                }

                let group_id = if let Some(group_id) = self.get_group_by_hash(&topic).await {
                    group_id
                } else {
                    let Ok(Response::Group((group_id,group_info))) =
                        client.request(peer_id, Request::Group(topic.clone())).await
                    else {
                        return Err(NetworkError::RequestError("error occured when request group info".to_string()));
                    };
                    self.add_group(group_id.clone(), group_info).await;
                    group_id
                };

                self.add_subscribe(&group_id, peer_id).await;
            }
            InboundEvent::Unsubscribed { peer_id, topic } => {
                if let Some(group_id) = self.get_group_by_hash(&topic).await {
                    self.remove_subscribe(&group_id, &peer_id).await;
                }
            }
            _ => {}
        }
        Ok(())
    }
}
