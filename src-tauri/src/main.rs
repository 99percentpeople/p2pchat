#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod error;
mod eventloop;
mod models;
mod network;
pub mod store;

use anyhow::Context;
use tauri::Manager;
use tokio::{sync::mpsc, task::LocalSet};

use eventloop::{
    command::{AppCommand, CommandHandle},
    FileShareApp,
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

                let app = FileShareApp {
                    app: app.handle(),
                    command_handle: command_handle1,
                    command_receiver: process_receiver,
                };

                tokio::spawn(app.run());
                Ok(())
            })
            .manage(command_handle)
            .invoke_handler(tauri::generate_handler![
                eventloop::get_file,
                eventloop::start_listen,
                eventloop::stop_listen,
                eventloop::listeners,
                eventloop::load_setting,
                eventloop::save_setting,
                eventloop::list_provide,
                eventloop::dial,
                eventloop::groups,
                eventloop::subscribe,
                eventloop::unsubscribe,
                eventloop::publish,
                eventloop::new_group,
                eventloop::local_peer_id,
                eventloop::connected_peers,
            ])
            .run(tauri::generate_context!())
            .with_context(|| "error while running tauri application")
    });

    local.await;
    Ok(())
}
