<template>
  <v-layout full-height class="h-100">
    <v-list lines="three" min-width="100%">
      <v-list-subheader>界面设置</v-list-subheader>
      <v-list-item value="widgets" :active="false" @click="toggleTheme">
        <v-list-item-title>主题颜色</v-list-item-title>
        <v-list-item-subtitle>改变界面的主题颜色</v-list-item-subtitle>
        <template #append>
          <v-switch v-model="isDark" />
        </template>
      </v-list-item>
      <v-divider></v-divider>
      <v-list-subheader>传输设置</v-list-subheader>
      <v-list-item>
        <v-list-item-title>接收文件路径</v-list-item-title>
        <v-list-item-subtitle>设置接收文件的路径</v-list-item-subtitle>
        <v-list-item>
          <v-text-field
            v-model="setting.recvPath"
            density="compact"
            single-line
            variant="underlined"
          >
            <template #append>
              <v-btn @click="onChooseDir" color="primary" outlined dense
                >选择</v-btn
              >
            </template></v-text-field
          >
        </v-list-item>
        <v-btn @click="onSave" color="primary" outlined>保存</v-btn>
      </v-list-item>
      <v-divider></v-divider>
      <v-list-subheader>监听设置</v-list-subheader>
    </v-list>
  </v-layout>
</template>

<script setup lang="ts">
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/api/dialog";
import { downloadDir } from "@tauri-apps/api/path";
import { useTheme } from "vuetify";
import {
  startListen,
  stopListen,
  listeners,
  loadSetting,
  saveSetting,
} from "../utils/backend";

const theme = useTheme();
const isDark = computed(() => theme.global.name.value === "dark");

const toggleTheme = () =>
  (theme.global.name.value = theme.global.current.value.dark
    ? "light"
    : "dark");
let setting = reactive({
  recvPath: "",
});
onBeforeMount(async () => {
  setting.recvPath = await downloadDir();
  await onLoad();
});

function onChooseDir() {
  open({
    multiple: false,
    directory: true,
  }).then((res) => {
    if (typeof res == "string") {
      setting.recvPath = res;
    }
  });
}

async function onSave() {
  try {
    await saveSetting(setting);
  } catch (err) {
    console.log(err);
  }
}

async function onLoad() {
  Object.assign(setting, await loadSetting());
}

const listenStatus: {
  status: boolean;
  listeners: string[];
} = reactive({
  status: false,
  listeners: [],
});

let unlisten: null | (() => void) = null;

onUnmounted(() => {
  unlisten?.();
});
</script>

<style lang="scss">
.el-row:not(:last-child) {
  margin-bottom: 20px;
}
</style>
