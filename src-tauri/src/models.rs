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
pub enum UserStatus {
    #[default]
    Online,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    name: String,
    avatar: Option<Url>,
    #[serde(skip_deserializing)]
    status: UserStatus,
}

impl Default for UserInfo {
    fn default() -> Self {
        Self {
            name: "Anonymous".to_string(),
            avatar: None,
            status: UserStatus::Online,
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
#[derive(Debug, Clone)]
pub struct GroupManager {
    group_status: Arc<Mutex<HashMap<GroupId, GroupState>>>,
    groups: Arc<Mutex<HashMap<GroupId, GroupInfo>>>,
}

impl GroupManager {
    pub fn new() -> Self {
        Self {
            group_status: Arc::new(Mutex::new(HashMap::new())),
            groups: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    pub async fn add_group(&self, group_id: GroupId, group_info: GroupInfo) {
        self.groups
            .lock()
            .await
            .insert(group_id.clone(), group_info);
        self.group_status
            .lock()
            .await
            .insert(group_id, GroupState::new());
    }
    pub async fn remove_group(&self, group_id: &GroupId) {
        self.groups.lock().await.remove(group_id);
        self.group_status.lock().await.remove(group_id);
    }
    pub async fn get_groups(&self) -> HashMap<GroupId, GroupInfo> {
        self.groups.lock().await.clone()
    }
    pub async fn add_message<G: AsRef<GroupId>>(&self, group_id: G, message: GroupMessage) {
        if let Some(group_status) = self.group_status.lock().await.get_mut(group_id.as_ref()) {
            group_status.history.push(message);
        }
    }
    pub async fn get_group_info(&self, group_id: &GroupId) -> Option<GroupInfo> {
        self.groups.lock().await.get(group_id).cloned()
    }
    pub async fn get_group_status(&self, group_id: &GroupId) -> Option<GroupState> {
        self.group_status.lock().await.get(group_id).cloned()
    }
    pub async fn has_group(&self, group_id: &GroupId) -> bool {
        self.groups.lock().await.contains_key(group_id.as_ref())
    }
    pub async fn has_group_by_hash(&self, topic_hash: &TopicHash) -> bool {
        self.groups
            .lock()
            .await
            .keys()
            .any(|group_info| &group_info.topic().hash() == topic_hash)
    }
    pub async fn get_group_by_hash(&self, topic_hash: &TopicHash) -> Option<GroupId> {
        self.groups
            .lock()
            .await
            .keys()
            .find(|group_info| &group_info.topic().hash() == topic_hash)
            .cloned()
    }
    pub async fn is_group_exist(&self, group_id: &GroupId) -> bool {
        self.groups.lock().await.contains_key(group_id)
    }
    pub async fn add_subscribe(&self, group_id: &GroupId, peer_id: PeerId) {
        if let Some(group_status) = self.group_status.lock().await.get_mut(group_id) {
            group_status.subscribers.insert(peer_id);
        }
    }
    pub async fn remove_subscribe(&self, group_id: &GroupId, peer_id: &PeerId) {
        if let Some(group_status) = self.group_status.lock().await.get_mut(group_id) {
            group_status.subscribers.remove(peer_id);
        }
    }
}
#[derive(Debug, Clone)]
pub struct UserManager {
    users: Arc<Mutex<HashMap<PeerId, UserInfo>>>,
    user_subscribe: Arc<Mutex<HashMap<PeerId, HashSet<GroupId>>>>,
}

impl UserManager {
    pub fn new() -> Self {
        Self {
            users: Arc::new(Mutex::new(HashMap::new())),
            user_subscribe: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    pub async fn add_user(&self, peer_id: PeerId, user_info: UserInfo) {
        self.users.lock().await.insert(peer_id, user_info);
    }
    pub async fn remove_user(&self, peer_id: &PeerId) {
        self.users.lock().await.remove(peer_id);
    }
    async fn add_subscribe(&self, peer_id: PeerId, group_id: GroupId) -> Result<(), ManagerError> {
        if !self.users.lock().await.contains_key(&peer_id) {
            return Err(ManagerError::PeerNotExist(peer_id));
        }
        self.user_subscribe
            .lock()
            .await
            .entry(peer_id)
            .or_default()
            .insert(group_id);
        Ok(())
    }
    async fn remove_subscribe(&self, peer_id: &PeerId, group_id: &GroupId) -> bool {
        if let Some(groups) = self.user_subscribe.lock().await.get_mut(peer_id) {
            groups.remove(group_id)
        } else {
            false
        }
    }
    pub async fn get_user_info(&self, peer_id: &PeerId) -> Option<UserInfo> {
        self.users.lock().await.get(peer_id).cloned()
    }
    pub async fn change_user_status(&self, peer_id: &PeerId, status: UserStatus) {
        if let Some(user_info) = self.users.lock().await.get_mut(peer_id) {
            user_info.status = status;
        }
    }
    pub async fn get_user_subscribe(&self, peer_id: &PeerId) -> Option<Vec<GroupId>> {
        self.user_subscribe
            .lock()
            .await
            .get(peer_id)
            .map(|groups| groups.iter().cloned().collect())
    }
    pub async fn has_user(&self, peer_id: &PeerId) -> bool {
        self.users.lock().await.contains_key(peer_id)
    }
}

#[derive(Debug, Clone)]
pub struct FileManager {
    files: Arc<Mutex<HashMap<PeerId, Vec<FileInfo>>>>,
    file_resources: Arc<Mutex<HashMap<PeerId, Vec<FileSource>>>>,
}

impl FileManager {
    pub fn new() -> Self {
        Self {
            files: Arc::new(Mutex::new(HashMap::new())),
            file_resources: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    pub async fn add_local_file<P: AsRef<Path>>(
        &self,
        peer_id: PeerId,
        path: P,
    ) -> Result<FileInfo, io::Error> {
        let path = path.as_ref();
        let file_info = FileInfo::from_path(path).await?;
        self.files
            .lock()
            .await
            .entry(peer_id)
            .or_insert_with(Vec::new)
            .push(file_info.clone());
        self.file_resources
            .lock()
            .await
            .entry(peer_id)
            .or_insert_with(Vec::new)
            .push(FileSource::Local(path.to_path_buf()));
        Ok(file_info)
    }
    pub async fn add_remote_file(&self, peer_id: PeerId, file_info: FileInfo) {
        self.files
            .lock()
            .await
            .entry(peer_id)
            .or_insert_with(Vec::new)
            .push(file_info);
        self.file_resources
            .lock()
            .await
            .entry(peer_id)
            .or_insert_with(Vec::new)
            .push(FileSource::Remote(peer_id));
    }
    pub async fn list_provide(&self, peer_id: &PeerId) -> Option<Vec<FileInfo>> {
        self.files
            .lock()
            .await
            .get(peer_id)
            .map(|files| files.clone())
    }
}
#[derive(Debug, Clone)]
pub struct Manager {
    group: GroupManager,
    user: UserManager,
    file: FileManager,
}

impl Manager {
    pub fn new() -> Self {
        Self {
            group: GroupManager::new(),
            user: UserManager::new(),
            file: FileManager::new(),
        }
    }
    pub fn group(&self) -> &GroupManager {
        &self.group
    }
    pub fn user(&self) -> &UserManager {
        &self.user
    }
    pub fn file(&self) -> &FileManager {
        &self.file
    }
    pub async fn get_user_not_subscribe(&self, peer_id: &PeerId) -> Option<Vec<GroupId>> {
        let user_subscribe = self.user.get_user_subscribe(peer_id).await?;

        let groups = self
            .group
            .get_groups()
            .await
            .keys()
            .filter(|group_id| !user_subscribe.contains(group_id))
            .cloned()
            .collect();
        Some(groups)
    }
    pub async fn subscribe(&self, peer_id: PeerId, group_id: GroupId) -> Result<(), ManagerError> {
        if !self.group.is_group_exist(&group_id).await {
            return Err(ManagerError::GroupNotExist(group_id));
        }
        self.group.add_subscribe(&group_id, peer_id).await;
        self.user.add_subscribe(peer_id.clone(), group_id).await
    }
    pub async fn unsubscribe(&self, peer_id: &PeerId, group_id: &GroupId) -> bool {
        self.user.remove_subscribe(peer_id, group_id).await
    }
}
