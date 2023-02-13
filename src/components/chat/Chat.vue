<template>
  <div class="chat-layout" v-if="chatStatus === 'active'">
    <v-virtual-scroll
      :items="groupStatus!.history"
      item-key="timestamp"
      height="0"
      class="px-2"
      id="scroll"
    >
      <template #default="{ item, index }">
        <chat-message :self="local === item.source">
          <chat-content :msg="item.message" />
        </chat-message>
      </template>
    </v-virtual-scroll>
    <v-toolbar :elevation="8" class="px-4" color="background">
      <v-text-field
        variant="underlined"
        v-model="message"
        append-icon="mdi-send"
        @click:append="onSendText"
      >
      </v-text-field>
    </v-toolbar>
  </div>
  <v-container
    v-else-if="chatStatus === 'noactive'"
    class="h-100 d-flex justify-center align-center"
  >
    <v-icon icon="mdi-message-outline" size="150" color="grey" />
  </v-container>
  <v-container
    v-else-if="chatStatus === 'nojoin'"
    class="h-100 d-flex flex-column justify-center align-center text-grey"
  >
    <v-icon icon="mdi-account-plus" size="150" />
    <v-btn variant="tonal" color="info" @click="onJoin">加入</v-btn>
  </v-container>
  <v-container
    v-else-if="chatStatus === 'noother'"
    class="h-100 d-flex flex-column justify-center align-center text-grey"
  >
    <v-icon icon="mdi-account-voice" size="150" />
    <span class="text-subtitle-1">等待其他用户</span>
  </v-container>
</template>
<script setup lang="ts">
import { listen } from "@tauri-apps/api/event";
import { VVirtualScroll } from "vuetify/labs/VVirtualScroll";
import { getGroupStatus, localPeerId, subscribe } from "../../utils/backend";
import { GroupId, GroupInfo, GroupMessage } from "../../utils/types";
const props = defineProps<{
  groupId: GroupId | null;
}>();

onMounted(async () => {
  listen<[GroupId, GroupMessage]>("message", (event) => {
    if (event.payload[0] === props.groupId) {
      let message = event.payload[1];
      groupStatus.value?.history.push(message);
    }
  });
  listen<[GroupId, GroupInfo]>("group-update", (event) => {
    if (event.payload[0] === props.groupId) {
    }
  });
});

let groupStatus = computedAsync(async () => {
  if (!props.groupId) return null;
  return await getGroupStatus(props.groupId);
}, null);
let local = useAsyncState(async () => await localPeerId(), null);
const chatStatus = computed(() => {
  if (!groupStatus.value || !local.state.value) {
    return "noactive";
  }
  if (!groupStatus.value.subscribers.includes(local.state.value)) {
    return "nojoin";
  }
  if (groupStatus.value.subscribers.length < 2) {
    return "noother";
  }
  return "active";
});

function onSendText() {
  if (message.value == "") return;

  message.value = "";
}
function onJoin() {
  if (props.groupId) {
    subscribe(props.groupId);
  }
}
let message = ref("");
</script>
<style scoped lang="scss">
.chat-layout {
  display: flex;
  flex-direction: column;
  height: 100%;
}
</style>
