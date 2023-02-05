import { invoke } from "@tauri-apps/api";

export type FileInfo = {
  name: string;
  size: number;
  createTime?: Date;
  modifyTime?: Date;
};
export type FileSource = {
  local?: string;
  remote?: string;
};
export type Setting = {
  recvPath: string;
};

export type Group = string;

export type GroupMessage = {
  message: Message;
  timestamp: Date;
  source?: string;
};

export type Message = {
  text?: string;
  file?: FileInfo;
};

export type GroupInfo = {
  name: string;
  history: GroupMessage[];
  subscribers: string[];
};

export async function startListen(listenAddr?: string) {
  try {
    await invoke<string>("start_listen", {
      listenAddr,
    });
  } catch (err) {
    console.error(err);
    throw err;
  }
}

export async function stopListen() {
  try {
    await invoke("stop_listen");
  } catch (err) {
    console.error(err);
  }
}

export async function listeners(): Promise<string[]> {
  try {
    let listenAddr = await invoke<string[]>("listeners");
    return listenAddr;
  } catch (err) {
    console.error(err);
    throw err;
  }
}

export async function getFile(file: FileInfo) {
  await invoke("get_file", {
    file,
  }).catch((err) => {
    console.error(err);
  });
}
export async function startProvide(
  path: string,
  file?: FileInfo
): Promise<FileInfo> {
  try {
    const resfile = await invoke<FileInfo>("start_provide", { path, file });
    return resfile;
  } catch (err) {
    console.error(err);
    throw err;
  }
}

export async function stopProvide(file: FileInfo) {
  try {
    await invoke("stop_provide", { file });
  } catch (err) {
    console.error(err);
    throw err;
  }
}

export async function loadSetting(loadPath?: string) {
  try {
    let setting = await invoke("load_setting", { loadPath });
    return setting;
  } catch (err) {
    console.error(err);
    throw err;
  }
}

export async function saveSetting(setting: Setting, savePath?: string) {
  try {
    await invoke("save_setting", { setting, savePath });
  } catch (err) {
    console.error(err);
    throw err;
  }
}

export async function listProvide(): Promise<FileInfo[]> {
  try {
    let providers = await invoke<FileInfo[]>("list_provide");
    return providers;
  } catch (err) {
    console.error(err);
    throw err;
  }
}

export async function connectedPeers(): Promise<string[]> {
  try {
    let peers = await invoke<string[]>("connected_peers");
    return peers;
  } catch (err) {
    console.error(err);
    throw err;
  }
}

export async function dial(addr: string) {
  try {
    await invoke("dial", { addr });
  } catch (err) {
    console.error(err);
    throw err;
  }
}

export async function publish(
  group: Group,
  message: Message
): Promise<GroupMessage> {
  try {
    return await invoke("publish", { group, message });
  } catch (err) {
    console.error(err);
    throw err;
  }
}

export async function subscribe(group: Group): Promise<GroupInfo> {
  try {
    return await invoke("subscribe", { group });
  } catch (err) {
    console.error(err);
    throw err;
  }
}

export async function groups(): Promise<{ [index: string]: GroupInfo }> {
  return await invoke<{ [index: string]: GroupInfo }>("groups");
}

export async function newGroup(groupName: string): Promise<[Group, GroupInfo]> {
  return await invoke<[Group, GroupInfo]>("new_group", { groupName });
}

export async function localPeerId(): Promise<string> {
  return await invoke<string>("local_peer_id");
}
