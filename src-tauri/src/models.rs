use crate::{
    error::{ManagerError, SettingError, SettingErrorKind},
    network::message::Message,
};
use chrono::{DateTime, Utc};
use derive_more::Display;
use libp2p::{
    gossipsub::{Sha256Topic, TopicHash},
    PeerId,
};
use mediatype::MediaTypeBuf;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    path::{Path, PathBuf},
};
use std::{fmt::Display, sync::Arc};
use tokio::{
    fs,
    io::{self, AsyncReadExt, AsyncWriteExt},
    sync::Mutex,
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

// #[derive(Debug, Clone)]
// pub struct FileManager {
//     files: Arc<Mutex<HashMap<PeerId, Vec<FileInfo>>>>,
//     file_resources: Arc<Mutex<HashMap<PeerId, Vec<FileSource>>>>,
// }

// impl FileManager {
//     pub fn new() -> Self {
//         Self {
//             files: Arc::new(Mutex::new(HashMap::new())),
//             file_resources: Arc::new(Mutex::new(HashMap::new())),
//         }
//     }
//     pub async fn add_local_file<P: AsRef<Path>>(
//         &self,
//         peer_id: PeerId,
//         path: P,
//     ) -> Result<FileInfo, io::Error> {
//         let path = path.as_ref();
//         let file_info = FileInfo::from_path(path).await?;
//         self.files
//             .lock()
//             .await
//             .entry(peer_id)
//             .or_insert_with(Vec::new)
//             .push(file_info.clone());
//         self.file_resources
//             .lock()
//             .await
//             .entry(peer_id)
//             .or_insert_with(Vec::new)
//             .push(FileSource::Local(path.to_path_buf()));
//         Ok(file_info)
//     }
//     pub async fn add_remote_file(&self, peer_id: PeerId, file_info: FileInfo) {
//         self.files
//             .lock()
//             .await
//             .entry(peer_id)
//             .or_insert_with(Vec::new)
//             .push(file_info);
//         self.file_resources
//             .lock()
//             .await
//             .entry(peer_id)
//             .or_insert_with(Vec::new)
//             .push(FileSource::Remote(peer_id));
//     }
//     pub async fn list_provide(&self, peer_id: &PeerId) -> Option<Vec<FileInfo>> {
//         self.files
//             .lock()
//             .await
//             .get(peer_id)
//             .map(|files| files.clone())
//     }
// }
// #[derive(Debug, Clone)]
// pub struct Manager {
//     group: GroupManager,
//     user: UserManager,
//     file: FileManager,
// }

// impl Manager {
//     pub fn new() -> Self {
//         Self {
//             group: GroupManager::new(),
//             user: UserManager::new(),
//             file: FileManager::new(),
//         }
//     }
//     pub fn group(&self) -> &GroupManager {
//         &self.group
//     }
//     pub fn user(&self) -> &UserManager {
//         &self.user
//     }
//     pub fn file(&self) -> &FileManager {
//         &self.file
//     }
//     pub async fn get_user_not_subscribe(&self, peer_id: &PeerId) -> Option<Vec<GroupId>> {
//         let user_subscribe = self.user.get_user_subscribe(peer_id).await?;

//         let groups = self
//             .group
//             .get_groups()
//             .await
//             .keys()
//             .filter(|group_id| !user_subscribe.contains(group_id))
//             .cloned()
//             .collect();
//         Some(groups)
//     }
//     pub async fn subscribe(&self, peer_id: PeerId, group_id: GroupId) -> Result<(), ManagerError> {
//         if !self.group.is_group_exist(&group_id).await {
//             return Err(ManagerError::GroupNotExist(group_id));
//         }
//         self.group.add_subscribe(&group_id, peer_id).await;
//         self.user.add_subscribe(peer_id.clone(), group_id).await
//     }
//     pub async fn unsubscribe(&self, peer_id: &PeerId, group_id: &GroupId) -> bool {
//         self.user.remove_subscribe(peer_id, group_id).await
//     }
// }
