use derive_more::From;
use std::{
    collections::{hash_map::Entry, HashMap},
    iter,
    sync::Arc,
};

use crate::{
    error::NetworkError,
    models::{FileInfo, FileSource, GroupId, GroupInfo, Setting},
    network::{
        message::{self, Message},
        Client,
    },
};
use libp2p::{self, multiaddr::Protocol, swarm::derive_prelude::ListenerId, Multiaddr, PeerId};
use tokio::{
    fs,
    io::AsyncWriteExt,
    sync::{mpsc, oneshot, Mutex},
};

use super::{frontend::FrontendEvent, AppState};

// #[derive(Debug)]
// pub enum AppCommand {
//     Dial {
//         addr: Multiaddr,
//         sender: oneshot::Sender<Result<(), NetworkError>>,
//     },
//     Get {
//         file: FileInfo,
//         sender: oneshot::Sender<Result<(), NetworkError>>,
//     },
//     StartListen {
//         listen_addr: Option<Multiaddr>,
//         sender: oneshot::Sender<Result<(), NetworkError>>,
//     },
//     StopListen {
//         sender: oneshot::Sender<Result<(), NetworkError>>,
//     },
//     Listeners {
//         sender: oneshot::Sender<Arc<Mutex<HashMap<ListenerId, Vec<Multiaddr>>>>>,
//     },
//     Setting {
//         sender: oneshot::Sender<Arc<Mutex<Setting>>>,
//     },
//     Groups {
//         sender: oneshot::Sender<HashMap<GroupId, GroupInfo>>,
//     },
//     Subscribe {
//         group: GroupId,
//         sender: oneshot::Sender<Result<(), NetworkError>>,
//     },
//     Unsubscribe {
//         group: GroupId,
//         sender: oneshot::Sender<Result<(), NetworkError>>,
//     },
//     Publish {
//         group: GroupId,
//         message: message::Message,
//         sender: oneshot::Sender<Result<GroupMessage, NetworkError>>,
//     },
//     NewGroup {
//         group_info: GroupInfo,
//         sender: oneshot::Sender<Result<GroupId, NetworkError>>,
//     },
//     LocalPeerId {
//         sender: oneshot::Sender<PeerId>,
//     },
//     ConnectedPeers {
//         sender: oneshot::Sender<Vec<PeerId>>,
//     },
//     Manager {
//         sender: oneshot::Sender<Manager>,
//     },
// }

#[derive(Debug, Clone)]
pub struct CommandHandle {
    client: Client,
    state: AppState,
}

impl CommandHandle {
    pub async fn listeners(&self) -> Vec<Multiaddr> {
        let listeners = self.client.listeners().await;
        listeners
            .values()
            .flatten()
            .cloned()
            .collect::<Vec<Multiaddr>>()
    }
    pub async fn start_listen(
        &mut self,
        listen_addr: Option<Multiaddr>,
    ) -> Result<(), NetworkError> {
        self.client
            .start_listening(listen_addr.unwrap_or_else(|| "/ip4/0.0.0.0/tcp/0".parse().unwrap()))
            .await?;
        Ok(())
    }
    pub async fn stop_listen(
        &mut self,
        listen_addr: Option<Multiaddr>,
    ) -> Result<(), NetworkError> {
        let listeners = self.client.listeners().await;
        let listener_id = if let Some(addr) = listen_addr {
            listeners.iter().find(|(_, x)| x.contains(&addr))
        } else {
            None
        }
        .map_or_else(
            || Box::new(listeners.keys()) as Box<dyn Iterator<Item = &ListenerId>>,
            |(id, _)| Box::new(iter::once(id)) as Box<dyn Iterator<Item = &ListenerId>>,
        )
        .cloned()
        .collect();
        self.client.stop_listening(listener_id).await?;
        Ok(())
    }
    pub async fn setting(&mut self) -> Setting {
        self.state.setting.lock().await.to_owned()
    }
    pub async fn dail(&mut self, addr: Multiaddr) -> Result<(), NetworkError> {
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
        &mut self,
        group_id: GroupId,
        message: Message,
    ) -> Result<(), NetworkError> {
        self.client.publish(group_id.topic(), message).await?;
        Ok(())
    }
    pub async fn new_group(
        &mut self,
        group_name: String,
        description: Option<String>,
    ) -> Result<GroupId, NetworkError> {
        let new_group_id = GroupId::new();
        let group_info = GroupInfo {
            name: group_name,
            description,
        };

        self.client
            .new_group(new_group_id.clone(), group_info)
            .await?;
        Ok(new_group_id)
    }
    pub async fn subscribe(&mut self, group_id: GroupId) -> Result<(), NetworkError> {
        self.client.subscribe(group_id.topic()).await?;
        Ok(())
    }
    pub async fn unsubscribe(&mut self, group_id: GroupId) -> Result<(), NetworkError> {
        self.client.unsubscribe(group_id.topic()).await?;
        Ok(())
    }
}

// impl CommandHandle {
//     pub fn new(state: AppState) -> Self {

//         Self {
//             client: ,
//             state,
//         }
//     }
// }
// impl CommandHandle {
//     pub async fn start_listen(
//         &mut self,
//         listen_addr: Option<Multiaddr>,
//     ) -> Result<(), NetworkError> {
//         let addr = listen_addr.unwrap_or_else(|| "/ip4/0.0.0.0/tcp/0".parse().unwrap());
//         self.client.start_listening(addr.clone()).await
//     }

//     pub async fn listeners(&self) -> Arc<Mutex<HashMap<ListenerId, Vec<Multiaddr>>>> {

//     }
//     pub async fn setting(&self) -> Arc<Mutex<Setting>> {
//         let (sender, receiver) = oneshot::channel();
//         self.0.send(AppCommand::Setting { sender }).await.unwrap();
//         receiver.await.unwrap()
//     }
//     pub async fn dial(&self, peer_addr: Multiaddr) -> Result<(), NetworkError> {
//         let (sender, receiver) = oneshot::channel();
//         self.0
//             .send(AppCommand::Dial {
//                 addr: peer_addr,
//                 sender,
//             })
//             .await
//             .unwrap();
//         receiver.await.unwrap()
//     }
//     pub async fn groups(&self) -> HashMap<GroupId, GroupInfo> {
//         let (sender, receiver) = oneshot::channel();
//         self.0.send(AppCommand::Groups { sender }).await.unwrap();
//         receiver.await.unwrap()
//     }
//     pub async fn subscribe(&self, group: GroupId) -> Result<(), NetworkError> {
//         let (sender, receiver) = oneshot::channel();
//         self.0
//             .send(AppCommand::Subscribe { group, sender })
//             .await
//             .unwrap();
//         receiver.await.unwrap()
//     }
//     pub async fn unsubscribe(&self, group: GroupId) -> Result<(), NetworkError> {
//         let (sender, receiver) = oneshot::channel();
//         self.0
//             .send(AppCommand::Unsubscribe { group, sender })
//             .await
//             .unwrap();
//         receiver.await.unwrap()
//     }
//     pub async fn publish(
//         &self,
//         group: GroupId,
//         message: message::Message,
//     ) -> Result<GroupMessage, NetworkError> {
//         let (sender, receiver) = oneshot::channel();
//         self.0
//             .send(AppCommand::Publish {
//                 group,
//                 message,
//                 sender,
//             })
//             .await
//             .unwrap();
//         receiver.await.unwrap()
//     }
//     pub async fn new_group(&self, group_info: GroupInfo) -> Result<GroupId, NetworkError> {
//         let (sender, receiver) = oneshot::channel();
//         self.0
//             .send(AppCommand::NewGroup { group_info, sender })
//             .await
//             .unwrap();
//         receiver.await.unwrap()
//     }
//     pub async fn local_peer_id(&self) -> PeerId {
//         let (sender, receiver) = oneshot::channel();
//         self.0
//             .send(AppCommand::LocalPeerId { sender })
//             .await
//             .unwrap();
//         receiver.await.unwrap()
//     }
//     pub async fn connected_peers(&self) -> Vec<PeerId> {
//         let (sender, receiver) = oneshot::channel();
//         self.0
//             .send(AppCommand::ConnectedPeers { sender })
//             .await
//             .unwrap();
//         receiver.await.unwrap()
//     }
//     pub async fn manager(&self) -> Manager {
//         let (sender, receiver) = oneshot::channel();
//         self.0.send(AppCommand::Manager { sender }).await.unwrap();
//         receiver.await.unwrap()
//     }

//     pub async fn dail(&self) -> Result<(), NetworkError> {}
// }

// impl CommandEventLoop {
//     pub async fn run(mut self) {
//         while let Some(command) = self.command_receiver.recv().await {
//             let state = self.state.clone();
//             let mut client = self.client.clone();
//             let frontend_sender = self.frontend_sender.clone();
//             tokio::spawn(async move {
//                 match command {
//                     AppCommand::Dial { addr, sender } => {
//                         let peer_id = match addr.iter().last() {
//                             Some(Protocol::P2p(hash)) => {
//                                 PeerId::from_multihash(hash).expect("Valid hash.")
//                             }
//                             _ => return log::error!("Expect peer multiaddr to contain peer ID."),
//                         };
//                         if let Err(e) = client.dial(peer_id).await {
//                             return sender.send(Err(e)).unwrap();
//                         }
//                         sender.send(Ok(())).unwrap();
//                     }
//                     AppCommand::Get { file, sender } => {
//                         // let res = match state.provide_list.lock().await.get(&file) {
//                         //     Some(FileSource::Remote(peer_id)) => {
//                         //         match client
//                         //             .request(peer_id.clone(), message::Request::File(file.clone()))
//                         //             .await
//                         //         {
//                         //             Ok(message::Response::File(file_content)) => {
//                         //                 let mut buffer = std::io::Cursor::new(file_content);
//                         //                 // Write the file to disk by given path.
//                         //                 let path =
//                         //                     state.setting.lock().await.recv_path.join(file.name);
//                         //                 let mut file = fs::OpenOptions::new()
//                         //                     .write(true)
//                         //                     .create(true)
//                         //                     .open(path)
//                         //                     .await
//                         //                     .unwrap();
//                         //                 file.write_all_buf(&mut buffer).await.unwrap();
//                         //                 Ok(())
//                         //             }
//                         //             Err(e) => Err(e),
//                         //             _ => Err(anyhow::anyhow!(
//                         //                 "Unexpected error occurred when requesting file {file:?}."
//                         //             )
//                         //             .into()),
//                         //         }
//                         //     }
//                         //     Some(FileSource::Local(_)) => Err(anyhow::anyhow!(
//                         //         "File {file:?} is already in local storage."
//                         //     )
//                         //     .into()),
//                         //     None => Err(anyhow::anyhow!(
//                         //         "Could not find provider for file {file:?}."
//                         //     )
//                         //     .into()),
//                         // };
//                         // sender.send(res).unwrap();
//                     }
//                     AppCommand::StartListen {
//                         listen_addr: listen_address,
//                         sender,
//                     } => {
//                         let addr =
//                             listen_address.unwrap_or_else(|| "/ip4/0.0.0.0/tcp/0".parse().unwrap());

//                         match client.start_listening(addr.clone()).await {
//                             Ok(_) => sender.send(Ok(())).unwrap(),
//                             Err(e) => sender.send(Err(e)).unwrap(),
//                         }
//                     }
//                     AppCommand::StopListen { sender } => {
//                         match client
//                             .stop_listening(state.listeners.lock().await.keys().cloned().collect())
//                             .await
//                         {
//                             Ok(_) => sender.send(Ok(())).unwrap(),
//                             Err(e) => sender.send(Err(e)).unwrap(),
//                         }
//                     }
//                     AppCommand::Setting { sender } => {
//                         sender.send(state.setting.clone()).unwrap();
//                     }
//                     AppCommand::ConnectedPeers { sender } => {
//                         sender.send(client.connected_peers().await).unwrap();
//                     }
//                     AppCommand::Listeners { sender } => {
//                         sender.send(state.listeners.clone()).unwrap();
//                     }
//                     AppCommand::Groups { sender } => {
//                         sender
//                             .send(state.manager.group().get_groups().await)
//                             .unwrap();
//                     }
//                     AppCommand::Subscribe { group, sender } => {
//                         if state.manager.group().has_group(&group).await {
//                             match client.subscribe(group.topic()).await {
//                                 Ok(_) => {
//                                     state
//                                         .manager
//                                         .subscribe(client.local_peer_id(), group.clone())
//                                         .await
//                                         .unwrap();

//                                     frontend_sender
//                                         .send(FrontendEvent::GroupUpdate {
//                                             group_info: state
//                                                 .manager
//                                                 .group()
//                                                 .get_group_info(&group)
//                                                 .await
//                                                 .unwrap(),
//                                             group_id: group,
//                                         })
//                                         .await
//                                         .unwrap();
//                                     sender.send(Ok(())).unwrap()
//                                 }
//                                 Err(err) => sender.send(Err(err)).unwrap(),
//                             };
//                         } else {
//                             sender
//                                 .send(Err(anyhow::anyhow!("Group not found.").into()))
//                                 .unwrap();
//                         }
//                     }
//                     AppCommand::Unsubscribe { group, sender } => {
//                         match client.unsubscribe(group.topic()).await {
//                             Ok(_) => {
//                                 state
//                                     .manager
//                                     .unsubscribe(&client.local_peer_id(), &group)
//                                     .await;
//                                 sender.send(Ok(())).unwrap()
//                             }
//                             Err(e) => sender.send(Err(e)).unwrap(),
//                         }
//                     }
//                     AppCommand::Publish {
//                         group,
//                         message,
//                         sender,
//                     } => {
//                         let group_message = GroupMessage::new(message, client.local_peer_id());
//                         if state.manager.group().has_group(&group).await {
//                             match client.publish(group.topic(), group_message.clone()).await {
//                                 Ok(_) => {
//                                     state
//                                         .manager
//                                         .group()
//                                         .add_message(&group, group_message.clone())
//                                         .await;
//                                     sender.send(Ok(group_message)).unwrap()
//                                 }
//                                 Err(err) => sender.send(Err(err)).unwrap(),
//                             };
//                         } else {
//                             sender
//                                 .send(Err(anyhow::anyhow!("Group not found.").into()))
//                                 .unwrap();
//                         }
//                     }
//                     AppCommand::NewGroup { group_info, sender } => {
//                         let group_id = GroupId::new();
//                         let peer_id = client.local_peer_id();
//                         match client.subscribe(group_id.topic()).await {
//                             Ok(_) => {
//                                 // if !state.manager.user().has_user(&peer_id).await {
//                                 //     state
//                                 //         .manager
//                                 //         .user()
//                                 //         .add_user(
//                                 //             peer_id.clone(),
//                                 //             state.setting.lock().await.user_info.clone(),
//                                 //         )
//                                 //         .await;
//                                 // }
//                                 // state
//                                 //     .manager
//                                 //     .group()
//                                 //     .add_group(group_id.clone(), group_info.clone())
//                                 //     .await;
//                                 // state
//                                 //     .manager
//                                 //     .subscribe(peer_id, group_id.clone())
//                                 //     .await
//                                 //     .unwrap();
//                                 frontend_sender
//                                     .send(FrontendEvent::GroupUpdate {
//                                         group_id: group_id.clone(),
//                                         group_info: group_info.clone(),
//                                     })
//                                     .await
//                                     .unwrap();
//                                 sender.send(Ok(group_id)).unwrap();
//                             }
//                             Err(e) => sender.send(Err(e)).unwrap(),
//                         }
//                     }
//                     AppCommand::LocalPeerId { sender } => {
//                         sender.send(client.local_peer_id()).unwrap()
//                     }
//                     AppCommand::Manager { sender } => sender.send(state.manager.clone()).unwrap(),
//                 }
//             });
//         }
//     }
// }
