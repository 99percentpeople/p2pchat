import { useDark, usePreferredDark } from "@vueuse/core";
import { defineStore } from "pinia";

export const useUISettings = defineStore("settings", () => {
  const isDark = useDark();
  const prefersDark = usePreferredDark();

  return { isDark, prefersDark };
});
