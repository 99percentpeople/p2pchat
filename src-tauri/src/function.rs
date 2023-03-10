use crate::{
    chat_app::{frontend::FrontendEvent, AppState},
    error::NetworkError,
    network::{message::InboundEvent, Client},
};
use async_trait::async_trait;
use dyn_clone::DynClone;
use tokio::sync::mpsc;

#[async_trait]
pub trait HandleInboundEvent: DynClone {
    async fn handle_event(
        &mut self,
        event: InboundEvent,
        client: Client,
        state: AppState,
        sender: mpsc::Sender<FrontendEvent>,
    ) -> Result<(), NetworkError>;
}

dyn_clone::clone_trait_object!(HandleInboundEvent);
#[async_trait]
pub trait HandleCommand: DynClone {
    async fn handle_command(&self, command: &str) -> Result<serde_json::Value, NetworkError>;
}

dyn_clone::clone_trait_object!(HandleCommand);

pub trait AppManager: HandleInboundEvent + HandleCommand + DynClone {
    fn name(&self) -> &'static str;
}

dyn_clone::clone_trait_object!(AppManager);
