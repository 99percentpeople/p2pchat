use crate::{
    chat_app::{frontend::FrontendEvent, AppState},
    error::{ManagerError, NetworkError},
    network::{message::InboundEvent, Client},
};
use async_trait::async_trait;
use dyn_clone::DynClone;
use tokio::sync::mpsc;

///
#[async_trait]
pub trait HandleInboundEvent: DynClone + Send + Sync {
    async fn handle_event(
        &mut self,
        event: InboundEvent,
        client: Client,
        state: AppState,
        sender: mpsc::Sender<FrontendEvent>,
    ) -> Result<(), NetworkError>;
}

#[async_trait]
pub trait Invoke: DynClone + Send + Sync {
    async fn invoke(
        &self,
        action: &str,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, ManagerError>;
}

pub trait AppManager: HandleInboundEvent + Invoke + DynClone + Send + Sync {
    fn name(&self) -> &'static str;
}

dyn_clone::clone_trait_object!(HandleInboundEvent);
dyn_clone::clone_trait_object!(Invoke);
dyn_clone::clone_trait_object!(AppManager);
