use async_trait::async_trait;

use crate::{
    error::NetworkError,
    network::{message::InboundEvent, Client},
};

#[async_trait]
pub trait HandleInboundEvent {
    async fn handle_event(
        &mut self,
        event: InboundEvent,
        client: Client,
    ) -> Result<(), NetworkError>;
}
