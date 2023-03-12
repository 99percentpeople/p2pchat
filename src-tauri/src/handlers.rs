use std::collections::HashMap;

use libp2p::{swarm::derive_prelude::ListenerId, Multiaddr, PeerId};

use crate::{
    chat_app::app_command::AppCommandHandle,
    error::NetworkError,
    models::{GroupId, GroupInfo, Setting},
    network::message::Message,
};

#[tauri::command]
pub async fn get_listeners(
    handle: tauri::State<'_, AppCommandHandle>,
) -> Result<HashMap<u64, Vec<Multiaddr>>, String> {
    let listeners = handle
        .get_listeners()
        .await
        .into_iter()
        .map(|(k, v)| (unsafe { std::mem::transmute::<ListenerId, u64>(k) }, v))
        .collect();
    Ok(listeners)
}
#[tauri::command]
pub async fn start_listen(
    handle: tauri::State<'_, AppCommandHandle>,
    listen_addr: Option<Multiaddr>,
) -> Result<u64, NetworkError> {
    handle.start_listen(listen_addr).await.map_or_else(
        |e| Err(e),
        |id| Ok(unsafe { std::mem::transmute::<ListenerId, u64>(id) }),
    )
}

#[tauri::command]
pub async fn stop_listen(
    handle: tauri::State<'_, AppCommandHandle>,
    listener_id: Option<u64>,
) -> Result<(), NetworkError> {
    handle
        .stop_listen(listener_id.map(|id| unsafe { std::mem::transmute::<u64, ListenerId>(id) }))
        .await
}
#[tauri::command]
pub async fn setting(handle: tauri::State<'_, AppCommandHandle>) -> Result<Setting, String> {
    Ok(handle.setting().await)
}
#[tauri::command]
pub async fn dail(
    handle: tauri::State<'_, AppCommandHandle>,
    addr: Multiaddr,
) -> Result<(), NetworkError> {
    handle.dial(addr).await
}
#[tauri::command]
pub async fn publish_message(
    handle: tauri::State<'_, AppCommandHandle>,
    group_id: GroupId,
    message: Message,
) -> Result<(), NetworkError> {
    handle.publish_message(group_id, message).await
}
#[tauri::command]
pub async fn new_group(
    handle: tauri::State<'_, AppCommandHandle>,
    group_info: GroupInfo,
) -> Result<GroupId, NetworkError> {
    handle.new_group(group_info).await
}
#[tauri::command]
pub async fn subscribe(
    handle: tauri::State<'_, AppCommandHandle>,
    group_id: GroupId,
) -> Result<(), NetworkError> {
    handle.subscribe(group_id).await
}
#[tauri::command]
pub async fn unsubscribe(
    handle: tauri::State<'_, AppCommandHandle>,
    group_id: GroupId,
) -> Result<(), NetworkError> {
    handle.unsubscribe(group_id).await
}

#[tauri::command]
pub async fn invoke_manager(
    handle: tauri::State<'_, AppCommandHandle>,
    name: String,
    action: String,
    params: Option<serde_json::Value>,
) -> Result<serde_json::Value, NetworkError> {
    handle.invoke_manager(name, action, params).await
}

#[tauri::command]
pub fn get_managers(
    handle: tauri::State<'_, AppCommandHandle>,
) -> Result<Vec<String>, NetworkError> {
    Ok(handle.get_managers())
}

#[tauri::command]
pub fn get_local_peer_id(
    handle: tauri::State<'_, AppCommandHandle>,
) -> Result<PeerId, NetworkError> {
    Ok(handle.get_local_peer_id())
}
