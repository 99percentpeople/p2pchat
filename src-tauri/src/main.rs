#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod error;
mod event;
mod models;
mod network;
pub mod store;

use anyhow::Context;
use tauri::Manager;
use tokio::{sync::mpsc, task::LocalSet};

use event::{
    command::{AppCommand, CommandHandle},
    AppState, ChatApp,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("debug"));
    let (process_sender, process_receiver) = mpsc::channel::<AppCommand>(100);
    let command_handle = CommandHandle::new(process_sender.clone());

    let local = LocalSet::new();
    let command_handle1 = command_handle.clone();
    local.spawn_local(async {
        tauri::Builder::default()
            .setup(move |app| {
                let window = app.get_window("main").unwrap();

                #[cfg(target_os = "macos")]
                apply_vibrancy(&window, NSVisualEffectMaterial::AppearanceBased, None, None)
                    .with_context(|| {
                        "Unsupported platform! 'apply_vibrancy' is only supported on macOS"
                    })?;

                #[cfg(target_os = "windows")]
                window_vibrancy::apply_mica(&window)
                    .map_err(|e| anyhow::anyhow!(e.to_string()))
                    .with_context(|| {
                        "Unsupported platform! 'apply_mica' is only supported on Windows"
                    })?;

                #[cfg(any(windows, target_os = "macos"))]
                window_shadows::set_shadow(&window, true).unwrap();
                let state = AppState::default();
                let app = ChatApp {
                    app: app.handle(),
                    state,
                    command_handle: command_handle1,
                    command_receiver: process_receiver,
                };

                tokio::spawn(app.run());
                Ok(())
            })
            .manage(command_handle)
            .invoke_handler(tauri::generate_handler![
                event::get_file,
                event::start_listen,
                event::stop_listen,
                event::listeners,
                event::load_setting,
                event::save_setting,
                event::list_provide,
                event::dial,
                event::get_groups,
                event::subscribe,
                event::unsubscribe,
                event::publish_text,
                event::publish_file,
                event::new_group,
                event::local_peer_id,
                event::connected_peers,
                event::get_group_status,
                event::get_group_include_peer,
                event::get_group_not_include_peer
            ])
            .run(tauri::generate_context!())
            .with_context(|| "error while running tauri application")
    });

    local.await;
    Ok(())
}
