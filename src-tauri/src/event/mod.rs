use std::sync::Arc;
use tauri::AppHandle;

use crate::{
    function::HandleInboundEvent, managers::group::GroupManager, models::Setting, network,
};
use tokio::{
    join,
    sync::{mpsc, oneshot, Mutex},
};

pub mod command;
pub mod frontend;
pub mod inbound;

use self::inbound::InboundEventLoop;
#[derive(Debug, Clone)]
pub struct AppState {
    pub(super) setting: Arc<Mutex<Setting>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            setting: Arc::new(Mutex::new(Setting::default())),
        }
    }
}

pub struct ChatApp {
    pub app: AppHandle,
    pub state: AppState,
}

impl ChatApp {
    pub async fn run(self) -> anyhow::Result<()> {
        let network = network::new(None)?;
        let (frontend_sender, frontend_receiver) = mpsc::channel(100);

        let state = self.state;
        let group_manager = GroupManager::new();

        let managers = vec![Box::new(group_manager.clone()) as Box<dyn HandleInboundEvent + Send>];

        let inbound_event_loop = InboundEventLoop {
            client: network.client.clone(),
            inbound_event_receiver: network.event_receiver,
            frontend_sender: frontend_sender.clone(),
            state: state.clone(),
            managers,
        };

        let (_, _) = join![
            tokio::spawn(network.event_loop.run()),
            tokio::spawn(inbound_event_loop.run()),
        ];

        Ok(())
    }
}
