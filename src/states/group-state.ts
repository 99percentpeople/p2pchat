import { getGroups, getGroupState } from "@/utils/backend";
import { AppEvent } from "@/utils/app-event";
import { defineStore } from "pinia";
import { GroupId, GroupState } from "@/utils/types";

export const useGroupState = defineStore("group", () => {
  const groups = useAsyncState(getGroups(), {}, { shallow: false });
  const groupStates = computedAsync(
    async () => {
      let newGroupStates = {} as { [index: GroupId]: GroupState };
      for (let groupId of Object.keys(groups.state.value)) {
        newGroupStates[groupId] = await getGroupState(groupId);
      }
      return newGroupStates;
    },
    {},
    { shallow: false }
  );

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

  return {
    groups: groups.state,
    groupStates: groupStates,
  };
});
