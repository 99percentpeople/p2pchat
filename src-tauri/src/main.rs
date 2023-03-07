#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod error;
mod event;
mod function;
mod managers;
mod models;
mod network;
pub mod store;

use anyhow::Context;
use tauri::Manager;
use tokio::{join, task::LocalSet};

use event::{AppState, ChatApp};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("debug"));

    let local = LocalSet::new();

    let tauri_app = tauri::Builder::default()
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

            Ok(())
        })
        .build(tauri::generate_context!())?;

    let app_state = AppState::default();

    let chat_app = ChatApp {
        app: tauri_app.handle(),
        state: app_state,
    };
    local.spawn_local(async {
        tauri_app.run(|_app_handle, event| match event {
            _ => {}
        })
    });

    let (_, _) = join!(local, chat_app.run());
    Ok(())
}
