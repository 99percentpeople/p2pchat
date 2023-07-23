use std::{fmt::Display, io, path::PathBuf};

use libp2p::{
    gossipsub::{PublishError, SubscriptionError},
    request_response::OutboundFailure,
    swarm::DialError,
    PeerId, TransportError,
};
use serde::Serialize;
use thiserror::Error;

use crate::models::GroupId;

#[derive(Debug, Error, Serialize)]
pub enum SettingErrorKind {
    #[error("invalid path: {0}")]
    InvalidPath(PathBuf),
}
#[derive(Debug, Error, Serialize)]
pub struct SettingError {
    kinds: Vec<SettingErrorKind>,
}

impl SettingError {
    pub fn new(kinds: Vec<SettingErrorKind>) -> Self {
        Self { kinds }
    }
    pub fn kinds(&self) -> impl Iterator<Item = &SettingErrorKind> {
        self.kinds.iter()
    }
}

impl Display for SettingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Setting error: {}",
            self.kinds()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("Listening error: {}", .0)]
    ListeningError(#[from] TransportError<io::Error>),
    #[error(transparent)]
    DialError(#[from] DialError),
    #[error("IO error: {0}")]
    IOError(#[from] io::Error),
    #[error(transparent)]
    PublishError(#[from] PublishError),
    #[error(transparent)]
    SubscriptionError(#[from] SubscriptionError),
    #[error("Request error: {0}")]
    RequestError(String),
    #[error(transparent)]
    SettingError(#[from] SettingError),
    #[error("Manager error: {0}")]
    ManagerError(#[from] ManagerError),
    #[error("invalid address: {0}")]
    InvalidAddress(String),
    #[error("command not found: {0}")]
    CommandNotFound(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<OutboundFailure> for NetworkError {
    fn from(value: OutboundFailure) -> Self {
        Self::RequestError(value.to_string())
    }
}

impl Serialize for NetworkError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

#[derive(Debug, Error)]
pub enum ManagerError {
    #[error("Group not exist {0}")]
    GroupNotExist(GroupId),
    #[error("Peer not exist {0}")]
    PeerNotExist(PeerId),
    #[error("invalid params: {0}")]
    InvalidParams(#[from] serde_json::Error),
    #[error("invalid action: {0}")]
    InvalidAction(String),
}
