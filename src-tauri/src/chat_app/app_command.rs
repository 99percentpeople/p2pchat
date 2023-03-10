use std::collections::HashMap;

use super::AppState;
use crate::{
    error::NetworkError,
    function::HandleCommand,
    models::{GroupId, GroupInfo, Setting},
    network::{message::Message, Client},
};
use libp2p::{self, multiaddr::Protocol, swarm::derive_prelude::ListenerId, Multiaddr, PeerId};

#[derive(Clone)]
pub struct AppCommandHandle {
    pub(crate) client: Client,
    pub(crate) state: AppState,
    pub(crate) managers: HashMap<String, Box<dyn HandleCommand + Send + Sync>>,
}

impl AppCommandHandle {
    pub async fn listeners(&self) -> HashMap<ListenerId, Vec<Multiaddr>> {
        self.client.listeners().await
    }
    pub async fn start_listen(
        &self,
        listen_addr: Option<Multiaddr>,
    ) -> Result<ListenerId, NetworkError> {
        self.client
            .start_listening(listen_addr.unwrap_or_else(|| "/ip4/0.0.0.0/tcp/0".parse().unwrap()))
            .await
    }
    pub async fn stop_listen(&self, listen_id: Option<ListenerId>) -> Result<(), NetworkError> {
        let listener_id = if let Some(listen_id) = listen_id {
            vec![listen_id]
        } else {
            self.client.listeners().await.keys().cloned().collect()
        };
        self.client.stop_listening(listener_id).await?;
        Ok(())
    }
    pub async fn setting(&self) -> Setting {
        self.state.setting.lock().await.to_owned()
    }
    pub async fn dial(&self, addr: Multiaddr) -> Result<(), NetworkError> {
        let peer_id = match addr.iter().last() {
            Some(Protocol::P2p(hash)) => PeerId::from_multihash(hash).expect("Valid hash."),
            _ => {
                return Err(NetworkError::InvalidAddress(
                    "Expect peer multiaddr to contain peer ID.".to_string(),
                ))
            }
        };
        self.client.dial(peer_id, addr).await
    }
    pub async fn publish_message(
        &self,
        group_id: GroupId,
        message: Message,
    ) -> Result<(), NetworkError> {
        self.client.publish(group_id.topic(), message).await?;
        Ok(())
    }
    pub async fn new_group(&self, group_info: GroupInfo) -> Result<GroupId, NetworkError> {
        let new_group_id = GroupId::new();

        self.client
            .new_group(new_group_id.clone(), group_info)
            .await?;
        Ok(new_group_id)
    }
    pub async fn subscribe(&self, group_id: GroupId) -> Result<(), NetworkError> {
        self.client.subscribe(group_id.topic()).await?;
        Ok(())
    }
    pub async fn unsubscribe(&self, group_id: GroupId) -> Result<(), NetworkError> {
        self.client.unsubscribe(group_id.topic()).await?;
        Ok(())
    }

    pub async fn manager(
        &self,
        manager: String,
        command: String,
    ) -> Result<serde_json::Value, NetworkError> {
        self.managers
            .get(&manager)
            .unwrap()
            .handle_command(&command)
            .await
    }
}
