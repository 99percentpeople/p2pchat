<template>
  <v-system-bar
    window
    data-tauri-drag-region
    class="titlebar"
    color="primary-darken-1"
  >
    <div class="titlebar-title">
      {{ appName }}
    </div>
    <div class="buttons">
      <v-hover>
        <template v-slot:default="{ isHovering, props }">
          <v-sheet
            v-bind="props"
            class="titlebar-button"
            @click="appWindow.minimize()"
            :color="isHovering ? 'primary' : 'primary-darken-1'"
          >
            <v-icon icon="mdi-window-minimize" />
          </v-sheet>
        </template>
      </v-hover>
      <v-hover>
        <template v-slot:default="{ isHovering, props }">
          <v-sheet
            v-bind="props"
            class="titlebar-button"
            @click="appWindow.toggleMaximize()"
            :color="isHovering ? 'primary' : 'primary-darken-1'"
          >
            <v-icon :icon="maximizeIcon" />
          </v-sheet>
        </template>
      </v-hover>
      <v-hover>
        <template v-slot:default="{ isHovering, props }">
          <v-sheet
            v-bind="props"
            class="titlebar-button titlebar-button-colse"
            @click="appWindow.close()"
            :color="isHovering ? 'red-darken-1' : 'primary-darken-1'"
          >
            <v-icon icon="mdi-window-close" />
          </v-sheet>
        </template>
      </v-hover>
    </div>
  </v-system-bar>
</template>

<script setup lang="ts">
import { appWindow } from "@tauri-apps/api/window";

let appName = ref(appWindow.label);

let input = ref("");
let maximizeIcon = ref("mdi-window-maximize");

let unlisten: null | (() => void) = null;
onMounted(async () => {
  unlisten = await appWindow.onResized(async () => {
    if (await appWindow.isMaximized()) {
      maximizeIcon.value = "mdi-dock-window";
    } else {
      maximizeIcon.value = "mdi-window-maximize";
    }
  });
});
onUnmounted(() => {
  unlisten?.();
});
</script>

<style scoped lang="scss">
@use "../styles/settings.scss";
.titlebar {
  user-select: none;
  display: flex;
  flex-wrap: nowrap;
  justify-content: space-between;
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
}
.buttons {
  height: 100%;
}
.titlebar-title {
  padding: 0 24px;
  line-height: var(--titlebar-area-height);
  pointer-events: none;
  font-size: 8pt;
}

.titlebar-button {
  display: inline-flex;
  justify-content: center;
  align-items: center;
  width: 45px;
  height: 100%;
  transition: all 0.1s ease;
}

.titlebar-button > img {
  transition: all 0.1s ease;
}

.titlebar-button:hover {
  // background-color: map-get(settings.$light-blue, base);
}

.titlebar-button-colse:hover {
  background: red;
}
</style>
