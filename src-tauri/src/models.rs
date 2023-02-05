use chrono::{DateTime, Utc};
use libp2p::{gossipsub::Sha256Topic, PeerId};
use mediatype::MediaTypeBuf;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    path::{Path, PathBuf},
};
use tokio::{
    fs,
    io::{self, AsyncReadExt, AsyncWriteExt},
};
use uuid::Uuid;

use crate::{
    error::{SettingError, SettingErrorKind},
    network::message::Message,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileInfo {
    pub name: String,
    pub size: u64,
    pub file_type: Option<MediaTypeBuf>,
}

impl PartialEq for FileInfo {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for FileInfo {}

impl Hash for FileInfo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl FileInfo {
    pub async fn from_path<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let data = fs::metadata(&path).await?;
        let path = path.as_ref();
        let file_type = mime_guess::from_path(path)
            .first()
            .map_or(None, |v| v.to_string().parse().ok());
        Ok(Self {
            name: path
                .file_name()
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid file name"))?
                .to_string_lossy()
                .to_string(),
            size: data.len(),
            file_type,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Setting {
    pub recv_path: PathBuf,
}

impl Setting {
    pub async fn save<P: AsRef<Path>>(&self, save_path: P) -> Result<(), io::Error> {
        let mut full_path = save_path.as_ref().to_path_buf();
        full_path.push(env!("CARGO_PKG_NAME"));
        full_path.set_extension("json");
        let buf = serde_json::to_vec(&self)?;
        let mut file = fs::File::create(full_path).await?;
        file.write_all(&buf).await?;
        Ok(())
    }
    pub async fn load<P: AsRef<Path>>(load_path: P) -> Result<Setting, io::Error> {
        let mut full_path = load_path.as_ref().to_path_buf();
        full_path.push(env!("CARGO_PKG_NAME"));
        full_path.set_extension("json");
        let mut file = fs::File::open(full_path).await?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).await?;
        let setting = serde_json::from_slice(&buf)?;
        Ok(setting)
    }
    pub fn merge(&mut self, other: Setting) -> Result<(), SettingError> {
        let mut kinds = Vec::new();
        if self.recv_path != other.recv_path {
            if !other.recv_path.exists() {
                kinds.push(SettingErrorKind::InvalidPath(other.recv_path))
            } else {
                self.recv_path = other.recv_path;
            }
        }
        if !kinds.is_empty() {
            Err(SettingError::new(kinds))
        } else {
            Ok(())
        }
    }
}

impl Default for Setting {
    fn default() -> Self {
        Self {
            recv_path: dirs::desktop_dir().unwrap_or_else(|| PathBuf::from(".")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct Group(Uuid);

impl Group {
    pub fn new(name: String) -> (Self, GroupInfo) {
        (Self(Uuid::new_v4()), GroupInfo::new(name))
    }

    pub fn topic(&self) -> Sha256Topic {
        Sha256Topic::new(self.0.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupInfo {
    pub name: String,
    pub history: Vec<GroupMessage>,
    pub subscribers: HashSet<PeerId>,
}

impl GroupInfo {
    pub fn contains_peer(&self, peer_id: &PeerId) -> bool {
        self.subscribers.contains(peer_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GroupMessage {
    pub source: Option<PeerId>,
    pub timestamp: DateTime<Utc>,
    pub message: Message,
}

impl GroupMessage {
    pub fn new(message: Message, source: Option<PeerId>) -> Self {
        Self {
            source,
            timestamp: Utc::now(),
            message,
        }
    }
}

impl GroupInfo {
    pub fn new(name: String) -> Self {
        Self {
            name,
            history: vec![],
            subscribers: HashSet::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileSource {
    Local(PathBuf),
    Remote(PeerId),
}

impl FileSource {
    pub fn is_local(&self) -> bool {
        matches!(self, Self::Local(_))
    }
    pub fn is_remote(&self) -> bool {
        matches!(self, Self::Remote(_))
    }
}

pub struct UserInfo {
    name: String,
    avatar: Option<MediaTypeBuf>,
}

pub struct GroupStatus {
    history: Vec<GroupMessage>,
    subscribers: HashMap<PeerId, UserInfo>,
}
