use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tauri::AppHandle;

use crate::{
    error::NetworkError,
    models::{FileInfo, Group, GroupInfo, GroupMessage, Setting},
    network::{self, message},
};
use libp2p::{self, Multiaddr, PeerId};
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

pub struct FileShareApp {
    pub app: AppHandle,
    pub command_handle: CommandHandle,
    pub command_receiver: mpsc::Receiver<AppCommand>,
}

impl FileShareApp {
    pub async fn run(self) -> anyhow::Result<()> {
        let network = network::new(None)?;
        let (frontend_sender, frontend_receiver) = mpsc::channel(100);

        let frontend_event_loop = FrontendEventLoop {
            app: self.app.clone(),
            frontend_receiver,
        };
        let inbound_event_loop = InboundEventLoop {
            client: network.client.clone(),
            inbound_event_receiver: network.event_receiver,
            command_handle: self.command_handle.clone(),
            frontend_sender: frontend_sender.clone(),
        };
        let command_event_loop = CommandEventLoop {
            command_receiver: self.command_receiver,
            client: network.client.clone(),
            provide_list: Arc::new(Mutex::new(HashMap::new())),
            setting: Arc::new(Mutex::new(Setting::default())),
            listeners: Arc::new(Mutex::new(HashMap::new())),
            group_list: Arc::new(Mutex::new(HashMap::new())),
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
    Ok(command_handle
        .provide_list()
        .await
        .lock()
        .await
        .keys()
        .cloned()
        .collect())
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
pub async fn groups(
    command_handle: tauri::State<'_, CommandHandle>,
) -> Result<HashMap<Group, GroupInfo>, NetworkError> {
    Ok(command_handle.groups().await.lock().await.clone())
}

#[tauri::command]
pub async fn subscribe(
    command_handle: tauri::State<'_, CommandHandle>,
    group: Group,
) -> Result<GroupInfo, NetworkError> {
    command_handle.subscribe(group).await
}

#[tauri::command]
pub async fn unsubscribe(
    command_handle: tauri::State<'_, CommandHandle>,
    group: Group,
) -> Result<(), NetworkError> {
    command_handle.unsubscribe(group).await?;
    Ok(())
}

#[tauri::command]
pub async fn publish(
    command_handle: tauri::State<'_, CommandHandle>,
    group: Group,
    message: message::Message,
) -> Result<GroupMessage, NetworkError> {
    command_handle.publish(group, message).await
}

#[tauri::command]
pub async fn new_group(
    command_handle: tauri::State<'_, CommandHandle>,
    group_name: String,
) -> Result<(Group, GroupInfo), NetworkError> {
    command_handle.new_group(group_name).await
}

#[tauri::command]
pub async fn local_peer_id(
    command_handle: tauri::State<'_, CommandHandle>,
) -> Result<PeerId, NetworkError> {
    Ok(command_handle.local_peer_id().await)
}
