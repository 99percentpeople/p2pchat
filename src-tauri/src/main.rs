#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
mod chat_app;
mod error;
mod managers;
mod models;
mod network;
mod plugin;
mod store;

use anyhow::Context;
use tauri::Manager;

use chat_app::ChatApp;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("debug"));

    tauri::Builder::default()
        .plugin(plugin::init())
        .setup(move |app| {
            let binding = app.windows();
            let window = binding.get("main").expect("failed to get window binding");
            #[cfg(target_os = "macos")]
            apply_vibrancy(&window, NSVisualEffectMaterial::AppearanceBased, None, None)
                .with_context(|| {
                    "Unsupported platform! 'apply_vibrancy' is only supported on macOS"
                })?;

            #[cfg(target_os = "windows")]
            window_vibrancy::apply_mica(&window, None)
                .map_err(|e| anyhow::anyhow!(e.to_string()))
                .with_context(|| {
                    "Unsupported platform! 'apply_mica' is only supported on Windows"
                })?;

            #[cfg(any(windows, target_os = "macos"))]
            window_shadows::set_shadow(&window, true).unwrap();

            let mut chat_app = ChatApp::new(app.app_handle()).initialize()?;
            let handle = chat_app.command_handle()?;
            tokio::spawn(async move {
                chat_app.run().await;
            });
            app.manage(handle);

            Ok(())
        })
        .run(tauri::generate_context!())?;

    Ok(())
}
