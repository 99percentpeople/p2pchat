use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tauri::AppHandle;

use crate::{
    error::{ManagerError, NetworkError},
    models::{FileInfo, GroupId, GroupInfo, GroupMessage, GroupState, Manager, Setting},
    network::{self, message},
};
use libp2p::{self, swarm::derive_prelude::ListenerId, Multiaddr, PeerId};
use tokio::{
    join,
    sync::{mpsc, oneshot, Mutex},
};

pub mod command;
pub mod frontend;
pub mod inbound;

use self::{
    command::{AppCommand, CommandEventLoop, CommandHandle},
    frontend::FrontendEventLoop,
    inbound::InboundEventLoop,
};
#[derive(Debug, Clone)]
pub struct AppState {
    pub(super) setting: Arc<Mutex<Setting>>,
    pub(super) listeners: Arc<Mutex<HashMap<ListenerId, Vec<Multiaddr>>>>,
    pub(super) manager: Manager,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            setting: Arc::new(Mutex::new(Setting::default())),
            listeners: Arc::new(Mutex::new(HashMap::new())),
            manager: Manager::new(),
        }
    }
}

pub struct ChatApp {
    pub app: AppHandle,
    pub state: AppState,
    pub command_handle: CommandHandle,
    pub command_receiver: mpsc::Receiver<AppCommand>,
}

impl ChatApp {
    pub async fn run(self) -> anyhow::Result<()> {
        let network = network::new(None)?;
        let (frontend_sender, frontend_receiver) = mpsc::channel(100);

        let state = self.state;

        let frontend_event_loop = FrontendEventLoop {
            app: self.app.clone(),
            state: state.clone(),
            frontend_receiver,
        };
        let inbound_event_loop = InboundEventLoop {
            client: network.client.clone(),
            inbound_event_receiver: network.event_receiver,
            frontend_sender: frontend_sender.clone(),
            state: state.clone(),
        };
        let command_event_loop = CommandEventLoop {
            command_receiver: self.command_receiver,
            client: network.client.clone(),
            state: state.clone(),
            frontend_sender,
        };

        let (_, _, _, _) = join![
            tokio::spawn(network.event_loop.run()),
            tokio::spawn(frontend_event_loop.run()),
            tokio::spawn(inbound_event_loop.run()),
            tokio::spawn(command_event_loop.run())
        ];

        Ok(())
    }
}

#[tauri::command]
pub async fn get_file(
    process_sender: tauri::State<'_, mpsc::Sender<AppCommand>>,
    file: FileInfo,
) -> Result<(), NetworkError> {
    let (sender, receiver) = oneshot::channel();
    process_sender
        .send(AppCommand::Get { file, sender })
        .await
        .expect("Command receiver not to be dropped.");
    receiver.await.expect("Sender not to be dropped.")
}

#[tauri::command]
pub async fn start_listen(
    command_handle: tauri::State<'_, CommandHandle>,
    listen_addr: Option<Multiaddr>,
) -> Result<(), NetworkError> {
    command_handle.start_listen(listen_addr).await
}

#[tauri::command]
pub async fn stop_listen(
    process_sender: tauri::State<'_, mpsc::Sender<AppCommand>>,
) -> Result<(), NetworkError> {
    let (sender, receiver) = oneshot::channel();
    process_sender
        .send(AppCommand::StopListen { sender })
        .await
        .expect("Command receiver not to be dropped.");
    receiver.await.expect("Sender not to be dropped.")
}

#[tauri::command]
pub async fn load_setting(
    command_handle: tauri::State<'_, CommandHandle>,
    load_path: Option<PathBuf>,
) -> Result<(), NetworkError> {
    let setting = command_handle.setting().await;
    let mut setting = setting.lock().await;
    let load_path = load_path.unwrap_or(".".into());
    setting.merge(Setting::load(load_path).await?)?;
    Ok(())
}

#[tauri::command]
pub async fn save_setting(
    command_handle: tauri::State<'_, CommandHandle>,
    setting: Setting,
    save_path: Option<PathBuf>,
) -> Result<(), NetworkError> {
    let orginal_setting = command_handle.setting().await;
    let mut orginal_setting = orginal_setting.lock().await;
    orginal_setting.merge(setting)?;
    let save_path = save_path.unwrap_or(".".into());
    orginal_setting.save(save_path).await?;
    Ok(())
}

#[tauri::command]
pub async fn list_provide(
    command_handle: tauri::State<'_, CommandHandle>,
) -> Result<Vec<FileInfo>, NetworkError> {
    let local_peer_id = command_handle.local_peer_id().await;
    command_handle
        .manager()
        .await
        .file()
        .list_provide(&local_peer_id)
        .await
        .ok_or(ManagerError::PeerNotExist(local_peer_id).into())
}

#[tauri::command]
pub async fn connected_peers(
    command_handle: tauri::State<'_, CommandHandle>,
) -> Result<Vec<PeerId>, NetworkError> {
    Ok(command_handle.connected_peers().await)
}

#[tauri::command]
pub async fn dial(
    command_handle: tauri::State<'_, CommandHandle>,
    peer_addr: Multiaddr,
) -> Result<(), NetworkError> {
    command_handle.dial(peer_addr).await
}

#[tauri::command]
pub async fn listeners(
    command_handle: tauri::State<'_, CommandHandle>,
) -> Result<Vec<Multiaddr>, NetworkError> {
    Ok(command_handle
        .listeners()
        .await
        .lock()
        .await
        .values()
        .flatten()
        .cloned()
        .collect())
}

#[tauri::command]
pub async fn get_groups(
    command_handle: tauri::State<'_, CommandHandle>,
) -> Result<HashMap<GroupId, GroupInfo>, NetworkError> {
    Ok(command_handle.groups().await)
}

#[tauri::command]
pub async fn subscribe(
    command_handle: tauri::State<'_, CommandHandle>,
    group: GroupId,
) -> Result<(), NetworkError> {
    command_handle.subscribe(group).await
}

#[tauri::command]
pub async fn unsubscribe(
    command_handle: tauri::State<'_, CommandHandle>,
    group: GroupId,
) -> Result<(), NetworkError> {
    command_handle.unsubscribe(group).await?;
    Ok(())
}

#[tauri::command]
pub async fn publish_text(
    command_handle: tauri::State<'_, CommandHandle>,
    group: GroupId,
    text: String,
) -> Result<GroupMessage, NetworkError> {
    let message = message::Message::Text(text);
    command_handle.publish(group, message).await
}

#[tauri::command]
pub async fn publish_file(
    command_handle: tauri::State<'_, CommandHandle>,
    group: GroupId,
    file_path: PathBuf,
) -> Result<GroupMessage, NetworkError> {
    let info = command_handle
        .manager()
        .await
        .file()
        .add_local_file(command_handle.local_peer_id().await, file_path)
        .await?;
    let message = message::Message::File(info);
    command_handle.publish(group, message).await
}

#[tauri::command]
pub async fn new_group(
    command_handle: tauri::State<'_, CommandHandle>,
    group_info: GroupInfo,
) -> Result<GroupId, NetworkError> {
    command_handle.new_group(group_info).await
}

#[tauri::command]
pub async fn local_peer_id(
    command_handle: tauri::State<'_, CommandHandle>,
) -> Result<PeerId, NetworkError> {
    Ok(command_handle.local_peer_id().await)
}

#[tauri::command]
pub async fn get_group_status(
    command_handle: tauri::State<'_, CommandHandle>,
    group_id: GroupId,
) -> Result<GroupState, String> {
    command_handle
        .manager()
        .await
        .group()
        .get_group_status(&group_id)
        .await
        .ok_or("Group not found".to_string())
}
#[tauri::command]
pub async fn get_group_include_peer(
    command_handle: tauri::State<'_, CommandHandle>,
    peer_id: PeerId,
) -> Result<Vec<GroupId>, ManagerError> {
    command_handle
        .manager()
        .await
        .user()
        .get_user_subscribe(&peer_id)
        .await
        .ok_or(ManagerError::PeerNotExist(peer_id))
}

#[tauri::command]
pub async fn get_group_not_include_peer(
    command_handle: tauri::State<'_, CommandHandle>,
    peer_id: PeerId,
) -> Result<Vec<GroupId>, ManagerError> {
    command_handle
        .manager()
        .await
        .get_user_not_subscribe(&peer_id)
        .await
        .ok_or(ManagerError::PeerNotExist(peer_id))
}
