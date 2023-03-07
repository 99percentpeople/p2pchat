use std::iter;

use crate::{
    function::HandleInboundEvent,
    models::{FileSource, GroupMessage, UserState},
    network::{
        message::{self, Response},
        Client,
    },
};

use futures::future;
use log::log;
use tokio::{fs, sync::mpsc};

use super::{frontend::FrontendEvent, AppState};

pub struct InboundEventLoop {
    pub(super) client: Client,
    pub(super) inbound_event_receiver: mpsc::Receiver<message::InboundEvent>,
    pub(super) frontend_sender: mpsc::Sender<FrontendEvent>,
    pub(super) state: AppState,
    pub(super) managers: Vec<Box<dyn HandleInboundEvent + Send>>,
}

impl InboundEventLoop {
    pub async fn run(mut self) {
        while let Some(event) = self.inbound_event_receiver.recv().await {
            for manager in self.managers.iter_mut() {
                match manager
                    .handle_event(event.clone(), self.client.clone())
                    .await
                {
                    Ok(_) => {}
                    Err(err) => log::error!("{err}"),
                }
            }
        }
    }
}
