use crate::{
    error::{SettingError, SettingErrorKind},
    network::message::Message,
};
use chrono::Utc;
use derive_more::Display;
use libp2p::{gossipsub::Sha256Topic, PeerId};
use mediatype::MediaTypeBuf;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    hash::Hash,
    path::{Path, PathBuf},
};
use tokio::{
    fs,
    io::{self, AsyncReadExt, AsyncWriteExt},
};
use url::Url;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileInfo {
    pub name: String,
    pub size: u64,
    pub file_type: Option<MediaTypeBuf>,
    pub hash: Option<String>,
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
            hash: None,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Setting {
    pub recv_path: PathBuf,
    pub user_info: UserInfo,
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
            user_info: UserInfo::default(),
        }
    }
}

#[derive(Debug, Clone, Display, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct GroupId(Uuid);

impl GroupId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn topic(&self) -> Sha256Topic {
        Sha256Topic::new(self.0.to_string())
    }
}
impl AsRef<GroupId> for &GroupId {
    fn as_ref(&self) -> &GroupId {
        &self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupInfo {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GroupMessage {
    pub source: PeerId,
    pub timestamp: i64,
    pub message: Message,
}

impl GroupMessage {
    pub fn new(message: Message, source: PeerId) -> Self {
        Self {
            source,
            timestamp: Utc::now().timestamp(),
            message,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum UserState {
    #[default]
    Online,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalUserInfo {
    pub peer_id: Option<PeerId>,
    pub name: String,
    pub avatar: Option<Url>,
}

impl From<LocalUserInfo> for UserInfo {
    fn from(info: LocalUserInfo) -> Self {
        Self {
            name: info.name,
            avatar: info.avatar,
            status: UserState::Online,
        }
    }
}

impl Default for LocalUserInfo {
    fn default() -> Self {
        Self {
            peer_id: None,
            name: "Anonymous".to_string(),
            avatar: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub name: String,
    pub avatar: Option<Url>,
    #[serde(skip_deserializing)]
    pub status: UserState,
}

impl UserInfo {
    pub fn new(name: String, avatar: Option<Url>) -> Self {
        Self {
            name,
            avatar,
            status: UserState::Online,
        }
    }
}

impl Default for UserInfo {
    fn default() -> Self {
        Self {
            name: "Anonymous".to_string(),
            avatar: None,
            status: UserState::Online,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupState {
    pub history: Vec<GroupMessage>,
    pub subscribers: HashSet<PeerId>,
}

impl GroupState {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            subscribers: HashSet::new(),
        }
    }
}
