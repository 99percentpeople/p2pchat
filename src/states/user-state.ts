import { getLocalPeerId, getUsers } from "@/utils/backend";
import { defineStore } from "pinia";
import { AppEvent } from "@/utils/app-event";
export const useUserState = defineStore("user", () => {
  const users = useAsyncState(getUsers(), {}, { shallow: false });
  const localPeerId = useAsyncState(getLocalPeerId(), null);
  AppEvent.onUserUpdate((event) => {
    users.state.value[event.payload[0]] = event.payload[1];
  });
  return {
    users: users.state,
    localPeerId: localPeerId.state,
  };
});
