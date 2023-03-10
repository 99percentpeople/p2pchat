#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
#![feature(trait_upcasting)]
mod chat_app;
mod error;
mod function;
mod handlers;
mod managers;
mod models;
mod network;
mod store;

use anyhow::Context;
use tauri::{generate_handler, Manager};
use tokio::{join, task::LocalSet};

use chat_app::ChatApp;

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
        .invoke_handler(generate_handler![
            handlers::listeners,
            handlers::start_listen,
            handlers::stop_listen,
            handlers::setting,
            handlers::dail,
            handlers::publish_message,
            handlers::new_group,
            handlers::subscribe,
            handlers::unsubscribe,
            handlers::manager,
        ])
        .build(tauri::generate_context!())?;

    let mut chat_app = ChatApp::new(tauri_app.handle());
    chat_app.initialize()?;
    tauri_app.manage(chat_app.command_handle()?);

    local.spawn_local(async {
        tauri_app.run(|_app_handle, event| match event {
            _ => {}
        })
    });

    let (_, _) = join!(local, tokio::spawn(chat_app.run()));
    Ok(())
}
