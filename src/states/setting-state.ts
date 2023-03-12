import { AppEvent } from "@/utils/app-event";
import { getListeners } from "@/utils/backend";
import { useDark, usePreferredDark } from "@vueuse/core";
import { defineStore } from "pinia";

export const useSettingState = defineStore("settings", () => {
  const isDark = useDark();
  const prefersDark = usePreferredDark();
  const listeners = useAsyncState(getListeners(), {}, { shallow: false });
  AppEvent.onListen((event) => {
    listeners.state.value[event.payload[0]] = event.payload[1];
  });
  return { isDark, prefersDark, listeners: listeners.state };
});
