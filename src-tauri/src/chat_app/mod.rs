use std::{collections::HashMap, sync::Arc};
use tauri::AppHandle;

use crate::{
    function::{AppManager, HandleInboundEvent, Invoke},
    managers::{group::GroupManager, user::UserManager},
    models::{LocalUserInfo, Setting},
    network::{self, EventLoop},
};
use tokio::{
    join,
    sync::{mpsc, Mutex},
};

pub mod app_command;
pub mod frontend;
pub mod inbound;

use self::{app_command::AppCommandHandle, frontend::FrontendEventLoop, inbound::InboundEventLoop};
#[derive(Debug, Clone)]
pub struct AppState {
    pub(super) setting: Arc<Mutex<Setting>>,
    pub(super) local_user: Arc<Mutex<LocalUserInfo>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            setting: Arc::new(Mutex::new(Setting::default())),
            local_user: Arc::new(Mutex::new(LocalUserInfo::default())),
        }
    }
}

pub struct ChatApp {
    pub app: AppHandle,
    pub state: AppState,
    pub client: Option<network::Client>,
    network_eventloop: Option<EventLoop>,
    inbound_eventloop: Option<InboundEventLoop>,
    frontend_eventloop: Option<FrontendEventLoop>,
    managers: HashMap<String, Box<dyn Invoke>>,
}

impl ChatApp {
    pub fn new(app: AppHandle) -> Self {
        Self {
            app,
            state: AppState::default(),
            client: None,
            network_eventloop: None,
            inbound_eventloop: None,
            frontend_eventloop: None,
            managers: HashMap::new(),
        }
    }

    pub fn initialize(&mut self) -> anyhow::Result<()> {
        let network = network::new(None)?;
        self.client = Some(network.client.clone());
        self.network_eventloop = Some(network.event_loop);
        let (frontend_sender, frontend_receiver) = mpsc::channel(100);

        let group = GroupManager::new();
        let user = UserManager::new();
        self.managers = [
            (
                group.name().to_string(),
                Box::new(group.clone()) as Box<dyn Invoke>,
            ),
            (
                user.name().to_string(),
                Box::new(user.clone()) as Box<dyn Invoke>,
            ),
        ]
        .into();

        self.inbound_eventloop = Some(InboundEventLoop {
            client: network.client.clone(),
            inbound_event_receiver: network.event_receiver,
            frontend_sender: frontend_sender.clone(),
            state: self.state.clone(),
            managers: vec![Box::new(group), Box::new(user)],
        });
        self.frontend_eventloop = Some(FrontendEventLoop {
            app: self.app.clone(),
            frontend_receiver,
            state: self.state.clone(),
        });
        Ok(())
    }
    pub async fn run(self) -> anyhow::Result<()> {
        let Some(network_eventloop) = self.network_eventloop else {
            anyhow::bail!("network event loop is not initialized");
        };
        let Some(inbound_event_loop) = self.inbound_eventloop else {
            anyhow::bail!("inbound event loop is not initialized");
        };
        let Some(frontend_eventloop) = self.frontend_eventloop else {
            anyhow::bail!("frontend event loop is not initialized");
        };
        let (_, _, _) = join![
            tokio::spawn(network_eventloop.run()),
            tokio::spawn(inbound_event_loop.run()),
            tokio::spawn(frontend_eventloop.run())
        ];
        Ok(())
    }
    pub fn command_handle(&self) -> anyhow::Result<AppCommandHandle> {
        let  Some(client) = &self.client else {
            anyhow::bail!("client is not initialized");
        };

        Ok(AppCommandHandle {
            client: client.clone(),
            state: self.state.clone(),
            managers: self.managers.clone(),
        })
    }
}
