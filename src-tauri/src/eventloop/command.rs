use derive_more::From;
use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};

use crate::{
    error::NetworkError,
    models::{FileInfo, FileSource, Group, GroupInfo, GroupMessage, Setting},
    network::{message, Client},
};
use libp2p::{self, multiaddr::Protocol, swarm::derive_prelude::ListenerId, Multiaddr, PeerId};
use tokio::{
    fs,
    io::AsyncWriteExt,
    sync::{mpsc, oneshot, Mutex},
};

#[derive(Debug)]
pub enum AppCommand {
    Dial {
        peer_addr: Multiaddr,
        sender: oneshot::Sender<Result<(), NetworkError>>,
    },
    Get {
        file: FileInfo,
        sender: oneshot::Sender<Result<(), NetworkError>>,
    },
    StartListen {
        listen_addr: Option<Multiaddr>,
        sender: oneshot::Sender<Result<(), NetworkError>>,
    },
    StopListen {
        sender: oneshot::Sender<Result<(), NetworkError>>,
    },
    Listeners {
        sender: oneshot::Sender<Arc<Mutex<HashMap<ListenerId, Vec<Multiaddr>>>>>,
    },
    Setting {
        sender: oneshot::Sender<Arc<Mutex<Setting>>>,
    },
    ProvideList {
        sender: oneshot::Sender<Arc<Mutex<HashMap<FileInfo, FileSource>>>>,
    },
    Groups {
        sender: oneshot::Sender<Arc<Mutex<HashMap<Group, GroupInfo>>>>,
    },
    Subscribe {
        group: Group,
        sender: oneshot::Sender<Result<GroupInfo, NetworkError>>,
    },
    Unsubscribe {
        group: Group,
        sender: oneshot::Sender<Result<(), NetworkError>>,
    },
    Publish {
        group: Group,
        message: message::Message,
        sender: oneshot::Sender<Result<GroupMessage, NetworkError>>,
    },
    NewGroup {
        group_name: String,
        sender: oneshot::Sender<Result<(Group, GroupInfo), NetworkError>>,
    },
    LocalPeerId {
        sender: oneshot::Sender<PeerId>,
    },
    ConnectedPeers {
        sender: oneshot::Sender<Vec<PeerId>>,
    },
}

pub struct CommandEventLoop {
    pub(super) client: Client,
    pub(super) command_receiver: mpsc::Receiver<AppCommand>,
    pub(super) setting: Arc<Mutex<Setting>>,
    pub(super) listeners: Arc<Mutex<HashMap<ListenerId, Vec<Multiaddr>>>>,
    pub(super) provide_list: Arc<Mutex<HashMap<FileInfo, FileSource>>>,
    pub(super) group_list: Arc<Mutex<HashMap<Group, GroupInfo>>>,
}

#[derive(Debug, Clone, From)]
pub struct CommandHandle(mpsc::Sender<AppCommand>);

impl CommandHandle {
    pub fn new(sender: mpsc::Sender<AppCommand>) -> Self {
        CommandHandle(sender)
    }

    pub async fn start_listen(&self, listen_addr: Option<Multiaddr>) -> Result<(), NetworkError> {
        let (sender, receiver) = oneshot::channel();
        self.0
            .send(AppCommand::StartListen {
                sender,
                listen_addr,
            })
            .await
            .unwrap();
        receiver.await.unwrap()?;
        Ok(())
    }
    pub async fn provide_list(&self) -> Arc<Mutex<HashMap<FileInfo, FileSource>>> {
        let (sender, receiver) = oneshot::channel();
        self.0
            .send(AppCommand::ProvideList { sender })
            .await
            .unwrap();
        receiver.await.unwrap()
    }

    pub async fn listeners(&self) -> Arc<Mutex<HashMap<ListenerId, Vec<Multiaddr>>>> {
        let (sender, receiver) = oneshot::channel();
        self.0.send(AppCommand::Listeners { sender }).await.unwrap();
        receiver.await.unwrap()
    }
    pub async fn setting(&self) -> Arc<Mutex<Setting>> {
        let (sender, receiver) = oneshot::channel();
        self.0.send(AppCommand::Setting { sender }).await.unwrap();
        receiver.await.unwrap()
    }
    pub async fn dial(&self, peer_addr: Multiaddr) -> Result<(), NetworkError> {
        let (sender, receiver) = oneshot::channel();
        self.0
            .send(AppCommand::Dial { peer_addr, sender })
            .await
            .unwrap();
        receiver.await.unwrap()
    }
    pub async fn groups(&self) -> Arc<Mutex<HashMap<Group, GroupInfo>>> {
        let (sender, receiver) = oneshot::channel();
        self.0.send(AppCommand::Groups { sender }).await.unwrap();
        receiver.await.unwrap()
    }
    pub async fn subscribe(&self, group: Group) -> Result<GroupInfo, NetworkError> {
        let (sender, receiver) = oneshot::channel();
        self.0
            .send(AppCommand::Subscribe { group, sender })
            .await
            .unwrap();
        receiver.await.unwrap()
    }
    pub async fn unsubscribe(&self, group: Group) -> Result<(), NetworkError> {
        let (sender, receiver) = oneshot::channel();
        self.0
            .send(AppCommand::Unsubscribe { group, sender })
            .await
            .unwrap();
        receiver.await.unwrap()
    }
    pub async fn publish(
        &self,
        group: Group,
        message: message::Message,
    ) -> Result<GroupMessage, NetworkError> {
        let (sender, receiver) = oneshot::channel();
        self.0
            .send(AppCommand::Publish {
                group,
                message,
                sender,
            })
            .await
            .unwrap();
        receiver.await.unwrap()
    }
    pub async fn new_group(&self, group_name: String) -> Result<(Group, GroupInfo), NetworkError> {
        let (sender, receiver) = oneshot::channel();
        self.0
            .send(AppCommand::NewGroup { group_name, sender })
            .await
            .unwrap();
        receiver.await.unwrap()
    }
    pub async fn local_peer_id(&self) -> PeerId {
        let (sender, receiver) = oneshot::channel();
        self.0
            .send(AppCommand::LocalPeerId { sender })
            .await
            .unwrap();
        receiver.await.unwrap()
    }
    pub async fn connected_peers(&self) -> Vec<PeerId> {
        let (sender, receiver) = oneshot::channel();
        self.0
            .send(AppCommand::ConnectedPeers { sender })
            .await
            .unwrap();
        receiver.await.unwrap()
    }
}

impl CommandEventLoop {
    pub async fn run(mut self) {
        while let Some(command) = self.command_receiver.recv().await {
            let mut client = self.client.clone();
            let provide_list = self.provide_list.clone();
            let setting = self.setting.clone();
            let listeners = self.listeners.clone();
            let group_list = self.group_list.clone();
            tokio::spawn(async move {
                match command {
                    AppCommand::Dial {
                        peer_addr: addr,
                        sender,
                    } => {
                        let peer_id = match addr.iter().last() {
                            Some(Protocol::P2p(hash)) => {
                                PeerId::from_multihash(hash).expect("Valid hash.")
                            }
                            _ => return log::error!("Expect peer multiaddr to contain peer ID."),
                        };
                        if let Err(e) = client.dial(peer_id).await {
                            return sender.send(Err(e)).unwrap();
                        }
                        sender.send(Ok(())).unwrap();
                    }
                    AppCommand::Get { file, sender } => {
                        let res = match provide_list.lock().await.get(&file) {
                            Some(FileSource::Remote(peer_id)) => {
                                match client
                                    .request(peer_id.clone(), message::Request::File(file.clone()))
                                    .await
                                {
                                    Ok(message::Response::File(file_content)) => {
                                        let mut buffer = std::io::Cursor::new(file_content);
                                        // Write the file to disk by given path.
                                        let path = setting.lock().await.recv_path.join(file.name);
                                        let mut file = fs::OpenOptions::new()
                                            .write(true)
                                            .create(true)
                                            .open(path)
                                            .await
                                            .unwrap();
                                        file.write_all_buf(&mut buffer).await.unwrap();
                                        Ok(())
                                    }
                                    Err(e) => Err(e),
                                    _ => Err(anyhow::anyhow!(
                                        "Unexpected error occurred when requesting file {file:?}."
                                    )
                                    .into()),
                                }
                            }
                            Some(FileSource::Local(_)) => Err(anyhow::anyhow!(
                                "File {file:?} is already in local storage."
                            )
                            .into()),
                            None => Err(anyhow::anyhow!(
                                "Could not find provider for file {file:?}."
                            )
                            .into()),
                        };
                        sender.send(res).unwrap();
                    }
                    AppCommand::StartListen {
                        listen_addr: listen_address,
                        sender,
                    } => {
                        let addr =
                            listen_address.unwrap_or_else(|| "/ip4/0.0.0.0/tcp/0".parse().unwrap());

                        match client.start_listening(addr.clone()).await {
                            Ok(_) => sender.send(Ok(())).unwrap(),
                            Err(e) => sender.send(Err(e)).unwrap(),
                        }
                    }
                    AppCommand::StopListen { sender } => {
                        match client
                            .stop_listening(listeners.lock().await.keys().cloned().collect())
                            .await
                        {
                            Ok(_) => sender.send(Ok(())).unwrap(),
                            Err(e) => sender.send(Err(e)).unwrap(),
                        }
                    }
                    AppCommand::Setting { sender } => {
                        sender.send(setting.clone()).unwrap();
                    }
                    AppCommand::ProvideList { sender } => {
                        sender.send(provide_list.clone()).unwrap();
                    }
                    AppCommand::ConnectedPeers { sender } => {
                        sender.send(client.connected_peers().await).unwrap();
                    }
                    AppCommand::Listeners { sender } => {
                        sender.send(listeners.clone()).unwrap();
                    }
                    AppCommand::Groups { sender } => {
                        sender.send(group_list.clone()).unwrap();
                    }
                    AppCommand::Subscribe { group, sender } => {
                        match group_list.lock().await.entry(group) {
                            Entry::Occupied(mut entry) => {
                                match client.subscribe(entry.key().topic()).await {
                                    Ok(_) => {
                                        let info = entry.get_mut();
                                        info.subscribers.insert(client.local_peer_id());
                                        sender.send(Ok(info.clone())).unwrap()
                                    }
                                    Err(err) => sender.send(Err(err)).unwrap(),
                                };
                            }
                            Entry::Vacant(_) => {
                                sender
                                    .send(Err(anyhow::anyhow!("Group not found.").into()))
                                    .unwrap();
                            }
                        }
                    }
                    AppCommand::Unsubscribe { group, sender } => {
                        match client.unsubscribe(group.topic()).await {
                            Ok(_) => sender.send(Ok(())).unwrap(),
                            Err(e) => sender.send(Err(e)).unwrap(),
                        }
                    }
                    AppCommand::Publish {
                        group,
                        message,
                        sender,
                    } => {
                        let group_message =
                            GroupMessage::new(message, Some(client.local_peer_id()));
                        match group_list.lock().await.entry(group) {
                            Entry::Occupied(mut entry) => {
                                match client
                                    .publish(entry.key().topic(), group_message.clone())
                                    .await
                                {
                                    Ok(_) => {
                                        let info = entry.get_mut();
                                        info.history.push(group_message.clone());
                                        sender.send(Ok(group_message)).unwrap()
                                    }
                                    Err(err) => sender.send(Err(err)).unwrap(),
                                };
                            }
                            Entry::Vacant(_) => {
                                sender
                                    .send(Err(anyhow::anyhow!("Group not found.").into()))
                                    .unwrap();
                            }
                        }
                    }
                    AppCommand::NewGroup { group_name, sender } => {
                        let (group, mut info) = Group::new(group_name);
                        info.subscribers.insert(client.local_peer_id());
                        match client.subscribe(group.topic()).await {
                            Ok(_) => {
                                group_list.lock().await.insert(group.clone(), info.clone());
                                sender.send(Ok((group, info))).unwrap();
                            }
                            Err(e) => sender.send(Err(e)).unwrap(),
                        }
                    }
                    AppCommand::LocalPeerId { sender } => {
                        sender.send(client.local_peer_id()).unwrap()
                    }
                }
            });
        }
    }
}
