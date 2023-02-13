use std::iter;

use crate::{
    models::{FileSource, GroupMessage, UserStatus},
    network::{
        message::{self, Response},
        Client,
    },
};

use futures::future;
use tokio::{fs, sync::mpsc};

use super::{frontend::FrontendEvent, AppState, CommandHandle};

#[derive(Debug)]
pub struct InboundEventLoop {
    pub(super) client: Client,
    pub(super) inbound_event_receiver: mpsc::Receiver<message::Event>,
    pub(super) frontend_sender: mpsc::Sender<FrontendEvent>,
    pub(super) state: AppState,
}

impl InboundEventLoop {
    pub async fn run(mut self) {
        while let Some(event) = self.inbound_event_receiver.recv().await {
            let mut client = self.client.clone();
            let frontend_sender = self.frontend_sender.clone();
            let state = self.state.clone();
            tokio::spawn(async move {
                match event {
                    message::Event::InboundRequest { request, channel } => {
                        log::debug!("inbound request {request:?}");
                        match request {
                            message::Request::File(file) => {
                                // match state.provide_list.lock().await.get(&file) {
                                // Some(FileSource::Local(path)) => {
                                //     let resp =
                                //         message::Response::File(fs::read(&path).await.unwrap());
                                //     client.response(resp, channel).await;
                                // }
                                // Some(FileSource::Remote(peer_id)) => {
                                //     match client
                                //         .request(peer_id.clone(), message::Request::File(file))
                                //         .await
                                //     {
                                //         Ok(Response::File(data)) => {
                                //             client
                                //                 .response(Response::File(data), channel)
                                //                 .await;
                                //         }
                                //         Ok(resp) => {
                                //             log::warn!("unexpected response {resp:?}");
                                //         }
                                //         Err(err) => {
                                //             log::warn!("request error {err}");
                                //         }
                                //     }
                                // }
                                // None => {
                                //     log::warn!("file not found {file:?}");
                                // }
                                // }
                            }
                            message::Request::Group(topic_hash) => {
                                if let Some(group) =
                                    state.manager.group().get_group_by_hash(&topic_hash).await
                                {
                                    let info =
                                        state.manager.group().get_group_info(&group).await.unwrap();
                                    client
                                        .response(message::Response::Group((group, info)), channel)
                                        .await;
                                } else {
                                    log::warn!("group not found {topic_hash:?}");
                                }
                            }
                            message::Request::User(peer_id) => {
                                if let Some(user) =
                                    state.manager.user().get_user_info(&peer_id).await
                                {
                                    client
                                        .response(message::Response::User(user), channel)
                                        .await;
                                } else {
                                    log::warn!("user not found {peer_id:?}");
                                }
                            }
                        }
                    }
                    message::Event::InboundMessage { message, .. } => {
                        let topic = message.topic;
                        let group_message: GroupMessage =
                            serde_json::from_slice(&message.data).unwrap();
                        if let Some(group_id) =
                            state.manager.group().get_group_by_hash(&topic).await
                        {
                            state
                                .manager
                                .group()
                                .add_message(&group_id, group_message.clone())
                                .await;
                            frontend_sender
                                .send(FrontendEvent::Message {
                                    group_id,
                                    message: group_message,
                                })
                                .await
                                .unwrap();
                        } else {
                            match client
                                .request(group_message.source, message::Request::Group(topic))
                                .await
                            {
                                Ok(message::Response::Group((group_id, info))) => {
                                    state
                                        .manager
                                        .group()
                                        .add_group(group_id.clone(), info)
                                        .await;
                                    state
                                        .manager
                                        .group()
                                        .add_message(&group_id, group_message.clone())
                                        .await;
                                    frontend_sender
                                        .send(FrontendEvent::Message {
                                            group_id,
                                            message: group_message,
                                        })
                                        .await
                                        .unwrap();
                                }
                                Err(e) => {
                                    log::error!("failed to get group info: {}", e);
                                }
                                _ => {
                                    log::error!("unexpected response");
                                }
                            }
                        }
                    }
                    message::Event::NewListenAddr {
                        address,
                        listener_id,
                    } => {
                        state
                            .listeners
                            .lock()
                            .await
                            .entry(listener_id)
                            .and_modify(|v| v.push(address.clone()))
                            .or_insert_with(|| vec![address.clone()]);

                        frontend_sender
                            .send(FrontendEvent::Listen {
                                listeners: state
                                    .listeners
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
                        state.listeners.lock().await.remove(&listener_id);
                        frontend_sender
                            .send(FrontendEvent::Listen {
                                listeners: state
                                    .listeners
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
                        if !state.manager.user().has_user(&peer_id).await {
                            match client
                                .request(peer_id.clone(), message::Request::User(peer_id.clone()))
                                .await
                            {
                                Ok(message::Response::User(user)) => {
                                    state.manager.user().add_user(peer_id, user).await;
                                }
                                Err(e) => {
                                    log::error!("failed to get user info: {}", e);
                                }
                                _ => {
                                    log::error!("unexpected response");
                                }
                            }
                        }
                        let group_id = if let Some(group_id) =
                            state.manager.group().get_group_by_hash(&topic).await
                        {
                            group_id
                        } else {
                            match client
                                .request(peer_id.clone(), message::Request::Group(topic))
                                .await
                            {
                                Ok(message::Response::Group((group_id, info))) => {
                                    state
                                        .manager
                                        .group()
                                        .add_group(group_id.clone(), info)
                                        .await;
                                    group_id
                                }
                                Err(e) => {
                                    log::error!("failed to get group info: {}", e);
                                    return;
                                }
                                _ => {
                                    log::error!("unexpected response");
                                    return;
                                }
                            }
                        };

                        state
                            .manager
                            .subscribe(peer_id, group_id.clone())
                            .await
                            .unwrap();

                        frontend_sender
                            .send(FrontendEvent::GroupUpdate {
                                group_info: state
                                    .manager
                                    .group()
                                    .get_group_info(&group_id)
                                    .await
                                    .unwrap(),
                                group_id,
                            })
                            .await
                            .unwrap();
                    }
                    message::Event::Unsubscribed { peer_id, topic } => {
                        let Some(group_id) = state.manager.group().get_group_by_hash(&topic).await else {
                            return ;
                        };
                        state.manager.unsubscribe(&peer_id, &group_id).await;
                    }
                    message::Event::PeerDiscovered { peer_id } => {
                        // state
                        //     .group_manager
                        //     .set_subscriber_status(&peer_id, UserStatus::Online)
                        //     .await;
                        // frontend_sender
                        //     .send(FrontendEvent::UserUpdate {
                        //         peer_id,
                        //         user_info: state
                        //             .group_manager
                        //             .get_user_info(&peer_id)
                        //             .await
                        //             .unwrap(),
                        //     })
                        //     .await
                        //     .unwrap();
                    }
                    message::Event::PeerExpired { peer_id } => {
                        // state
                        //     .group_manager
                        //     .set_subscriber_status(&peer_id, UserStatus::Offline)
                        //     .await;

                        // frontend_sender
                        //     .send(FrontendEvent::UserUpdate {
                        //         peer_id,
                        //         user_info: state
                        //             .group_manager
                        //             .get_user_info(&peer_id)
                        //             .await
                        //             .unwrap(),
                        //     })
                        //     .await
                        //     .unwrap();
                    }
                }
            });
        }
    }
}
