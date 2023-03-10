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
