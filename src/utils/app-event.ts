import { Event, listen } from "@tauri-apps/api/event";
import {
  GroupId,
  GroupInfo,
  GroupMessage,
  GroupState,
  Multiaddr,
  PeerId,
  UserInfo,
} from "./types";

export class AppEvent {
  static async onGroupUpdate(
    callBackFn: (args: Event<[GroupId, GroupInfo]>) => void
  ) {
    try {
      return await listen<[GroupId, GroupInfo]>("group-update", callBackFn);
    } catch (err) {
      console.error(err);
    }
  }
  static async onMessage(
    callBackFn: (args: Event<[GroupId, GroupMessage]>) => void
  ) {
    try {
      return await listen<[GroupId, GroupMessage]>("message", callBackFn);
    } catch (err) {
      console.error(err);
    }
  }
  static async onUserUpdate(
    callBackFn: (args: Event<[PeerId, UserInfo]>) => void
  ) {
    try {
      return await listen<[PeerId, UserInfo]>("user-update", callBackFn);
    } catch (err) {
      console.error(err);
    }
  }
  static async onSubscribed(
    callBackFn: (args: Event<[GroupId, PeerId]>) => void
  ) {
    try {
      return await listen<[GroupId, PeerId]>("subscribed", callBackFn);
    } catch (err) {
      console.error(err);
    }
  }
  static async onUnsubscribe(
    callBackFn: (args: Event<[GroupId, PeerId]>) => void
  ) {
    try {
      return await listen<[GroupId, PeerId]>("unsubscribe", callBackFn);
    } catch (err) {
      console.error(err);
    }
  }
  static async onListen(callBackFn: (args: Event<Multiaddr[]>) => void) {
    try {
      return await listen<Multiaddr[]>("listen", callBackFn);
    } catch (err) {
      console.error(err);
    }
  }
}
