import {
  getGroups,
  getLocalPeerId,
  getGroupState,
  getUsers,
} from "@/utils/backend";
import { AppEvent } from "@/utils/app-event";
import { defineStore } from "pinia";
import { GroupId, GroupState } from "@/utils/types";

export const useGlobal = defineStore("global", () => {
  let groups = useAsyncState(getGroups(), {}, { shallow: false });
  let groupStates = computedAsync<{ [index: GroupId]: GroupState }>(
    async () => {
      let newGroupStates = {} as { [index: GroupId]: GroupState };
      for (let groupId of Object.keys(groups.state.value)) {
        newGroupStates[groupId] = await getGroupState(groupId);
      }
      return newGroupStates;
    },
    {},
    { lazy: true, shallow: false }
  );
  let localPeerId = useAsyncState(getLocalPeerId(), null);
  let users = useAsyncState(getUsers(), {}, { shallow: false });
  AppEvent.onGroupUpdate((event) => {
    groups.state.value[event.payload[0]] = event.payload[1];
    console.log("group update", event.payload[0], event.payload[1]);
  });
  AppEvent.onSubscribed((event) => {
    console.log("subscribe", event.payload[0], event.payload[1]);

    groupStates.value[event.payload[0]].subscribers.push(event.payload[1]);
  });
  AppEvent.onUnsubscribe((event) => {
    groupStates.value[event.payload[0]].subscribers = groupStates.value[
      event.payload[0]
    ].subscribers.filter((id) => id !== event.payload[1]);
    if (groupStates.value[event.payload[0]].subscribers.length === 0) {
      delete groupStates.value[event.payload[0]];
    }
  });
  AppEvent.onMessage((event) => {
    groupStates.value[event.payload[0]].history.push(event.payload[1]);
  });
  AppEvent.onUserUpdate((event) => {
    users.state.value[event.payload[0]] = event.payload[1];
  });

  return {
    groups: groups.state,
    localPeerId: localPeerId.state,
    groupStates: groupStates,
    users: users.state,
  };
});
