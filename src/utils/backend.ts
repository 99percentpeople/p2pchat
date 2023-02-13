import { invoke } from "@tauri-apps/api";
import {
  FileInfo,
  GroupId,
  GroupInfo,
  GroupMessage,
  GroupStatus,
  Message,
  PeerId,
  Setting,
} from "./types";

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
  group: GroupId,
  message: Message
): Promise<GroupMessage> {
  try {
    return await invoke("publish", { group, message });
  } catch (err) {
    console.error(err);
    throw err;
  }
}

export async function subscribe(group: GroupId) {
  try {
    await invoke("subscribe", { group });
  } catch (err) {
    console.error(err);
    throw err;
  }
}

export async function getGroups(): Promise<{ [index: GroupId]: GroupInfo }> {
  return await invoke<{ [index: string]: GroupInfo }>("get_groups");
}

export async function newGroup(groupInfo: GroupInfo): Promise<GroupId> {
  return await invoke<GroupId>("new_group", { groupInfo });
}

export async function localPeerId(): Promise<string> {
  return await invoke<string>("local_peer_id");
}

export async function getGroupStatus(groupId: GroupId): Promise<GroupStatus> {
  return await invoke<GroupStatus>("get_group_status", { groupId });
}

export async function getGroupIncludePeer(peerId: PeerId): Promise<GroupId[]> {
  let res = await invoke<string[]>("get_group_include_peers", { peerId });
  console.log(res);

  return res;
}

export async function getGroupNotIncludePeer(
  peerId: PeerId
): Promise<GroupId[]> {
  return await invoke<string[]>("get_group_not_include_peers", { peerId });
}
