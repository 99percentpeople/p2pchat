<template>
  <div class="chat-layout" v-if="chatStatus === 'active'">
    <v-virtual-scroll
      :items="groupState!.history"
      item-key="timestamp"
      class="px-2 flex-grow-1"
      id="scroll"
    >
      <template #default="{ item, index }">
        <chat-message
          :user-info="users[item.source]"
          :self="localPeerId === item.source"
        >
          <chat-content :msg="item.message" />
        </chat-message>
      </template>
    </v-virtual-scroll>
    <div class="d-flex align-center px-2 pt-2 elevation-6">
      <v-textarea
        variant="filled"
        append-icon="mdi-send"
        auto-grow
        rows="1"
        max-rows="6"
        v-model="message"
        @click:append="onSendText"
      >
      </v-textarea>
    </div>
  </div>
  <v-container
    v-else-if="chatStatus === 'no-active'"
    class="h-100 d-flex justify-center align-center"
  >
    <v-icon icon="mdi-message-outline" size="150" color="grey" />
  </v-container>
  <v-container
    v-else-if="chatStatus === 'no-join'"
    class="h-100 d-flex flex-column justify-center align-center text-grey"
  >
    <v-icon icon="mdi-account-plus" size="150" />
    <v-btn variant="tonal" color="info" @click="onJoin">加入</v-btn>
  </v-container>
  <v-container
    v-else-if="chatStatus === 'no-other'"
    class="h-100 d-flex flex-column justify-center align-center text-grey"
  >
    <v-icon icon="mdi-account-voice" size="150" />
    <span class="text-subtitle-1">等待其他用户</span>
  </v-container>
</template>
<script setup lang="ts">
import { VVirtualScroll } from "vuetify/labs/VVirtualScroll";
import { publishMessage, subscribe } from "@/utils/backend";
import { GroupId } from "@/utils/types";
import { useGlobal } from "@/states/global";
const props = defineProps<{
  groupId: GroupId | null;
}>();
const global = useGlobal();
const { localPeerId, groupStates, users } = storeToRefs(global);

const groupState = computed(() => {
  if (props.groupId) {
    return groupStates.value[props.groupId];
  }
  return null;
});

const chatStatus = computed(() => {
  if (!groupState.value || !localPeerId.value) {
    return "no-active";
  }
  if (!groupState.value.subscribers.includes(localPeerId.value)) {
    return "no-join";
  }
  if (groupState.value.subscribers.length < 2) {
    return "no-other";
  }
  return "active";
});

function onSendText() {
  if (message.value == "") return;
  publishMessage(props.groupId!, { text: message.value });
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
