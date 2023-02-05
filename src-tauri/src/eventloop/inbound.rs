use crate::{
    models::{FileSource, GroupMessage},
    network::{message, Client},
};

use tokio::{fs, sync::mpsc};

use super::{frontend::FrontendEvent, CommandHandle};

#[derive(Debug)]
pub struct InboundEventLoop {
    pub(super) client: Client,
    pub(super) inbound_event_receiver: mpsc::Receiver<message::Event>,
    pub(super) command_handle: CommandHandle,
    pub(super) frontend_sender: mpsc::Sender<FrontendEvent>,
}

impl InboundEventLoop {
    pub async fn run(mut self) {
        while let Some(event) = self.inbound_event_receiver.recv().await {
            let command_handle = self.command_handle.clone();
            let mut client = self.client.clone();
            let frontend_sender = self.frontend_sender.clone();
            tokio::spawn(async move {
                match event {
                    message::Event::InboundRequest { request, channel } => {
                        log::debug!("inbound request {request:?}");
                        match request {
                            message::Request::File(file) => {
                                match command_handle.provide_list().await.lock().await.get(&file) {
                                    Some(FileSource::Local(path)) => {
                                        let resp =
                                            message::Response::File(fs::read(&path).await.unwrap());
                                        client.response(resp, channel).await;
                                    }
                                    Some(FileSource::Remote(peer_id)) => {
                                        let resp = client
                                            .request(peer_id.clone(), message::Request::File(file))
                                            .await
                                            .unwrap();
                                        client.response(resp, channel).await;
                                    }
                                    None => {
                                        log::warn!("file not found {file:?}");
                                    }
                                }
                            }
                            message::Request::Group(topic_hash) => {
                                if let Some((group, info)) = command_handle
                                    .groups()
                                    .await
                                    .lock()
                                    .await
                                    .iter()
                                    .find(|(group, _)| group.topic().hash() == topic_hash)
                                {
                                    client
                                        .response(
                                            message::Response::Group((group.clone(), info.clone())),
                                            channel,
                                        )
                                        .await;
                                } else {
                                    log::warn!("group not found {topic_hash:?}");
                                }
                            }
                        }
                    }
                    message::Event::InboundMessage {
                        propagation_source,
                        message_id,
                        message,
                    } => {
                        let topic = message.topic;
                        let group_message: GroupMessage =
                            serde_json::from_slice(&message.data).unwrap();
                        if let Some((group, info)) = command_handle
                            .groups()
                            .await
                            .lock()
                            .await
                            .iter_mut()
                            .find(|(group, _)| group.topic().hash() == topic)
                        {
                            info.history.push(group_message.clone());
                            frontend_sender
                                .send(FrontendEvent::Message {
                                    group: group.clone(),
                                    message: group_message,
                                })
                                .await
                                .unwrap();
                        };
                    }
                    message::Event::NewListenAddr {
                        address,
                        listener_id,
                    } => {
                        command_handle
                            .listeners()
                            .await
                            .lock()
                            .await
                            .entry(listener_id)
                            .and_modify(|v| v.push(address.clone()))
                            .or_insert_with(|| vec![address.clone()]);

                        frontend_sender
                            .send(FrontendEvent::Listen {
                                listeners: command_handle
                                    .listeners()
                                    .await
                                    .lock()
                                    .await
                                    .values()
                                    .flatten()
                                    .cloned()
                                    .collect(),
                            })
                            .await
                            .unwrap();
                    }
                    message::Event::ListenerClosed { listener_id, .. } => {
                        command_handle
                            .listeners()
                            .await
                            .lock()
                            .await
                            .remove(&listener_id);
                        frontend_sender
                            .send(FrontendEvent::Listen {
                                listeners: command_handle
                                    .listeners()
                                    .await
                                    .lock()
                                    .await
                                    .values()
                                    .flatten()
                                    .cloned()
                                    .collect(),
                            })
                            .await
                            .unwrap();
                    }
                    message::Event::Subscribed { peer_id, topic } => {
                        let groups = command_handle.groups().await;
                        let mut groups = groups.lock().await;
                        if let Some((_, info)) =
                            groups.iter_mut().find(|(g, _)| g.topic().hash() == topic)
                        {
                            info.subscribers.insert(peer_id);
                        } else {
                            match client
                                .request(peer_id, message::Request::Group(topic))
                                .await
                            {
                                Ok(message::Response::Group((group, info))) => {
                                    groups.insert(group, info);
                                }
                                Err(e) => {
                                    log::error!("failed to get group info: {}", e);
                                }
                                _ => {
                                    log::error!("unexpected response");
                                }
                            }
                        }
                        frontend_sender
                            .send(FrontendEvent::GroupUpdate {
                                group_list: groups.clone(),
                            })
                            .await
                            .unwrap();
                    }
                    message::Event::Unsubscribed { peer_id, topic } => {
                        let groups = command_handle.groups().await;
                        let mut groups = groups.lock().await;
                        if let Some(group) =
                            groups.keys().find(|g| g.topic().hash() == topic).cloned()
                        {
                            let info = groups.get_mut(&group).unwrap();
                            info.subscribers.retain(|p| p != &peer_id);
                            if info.subscribers.is_empty() {
                                groups.remove(&group);
                            }
                            frontend_sender
                                .send(FrontendEvent::GroupUpdate {
                                    group_list: groups.clone(),
                                })
                                .await
                                .unwrap();
                        }
                    }
                    message::Event::PeerDiscovered { peer_id } => {}
                }
            });
        }
    }
}
