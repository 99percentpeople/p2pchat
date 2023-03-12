<template>
  <div class="chat-layout">
    <v-virtual-scroll
      v-if="groupState"
      :visible-items="100"
      height="auto"
      :items="groupState.history"
      class="px-2 flex-grow-1"
      id="scroll"
    >
      <template #default="{ item }">
        <chat-message :peer-id="item.source" :user-info="users[item.source]">
          <span v-if="item.message.text">{{ item.message.text }} </span>
          <span v-else-if="item.message.file"> </span>
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
</template>

<script setup lang="ts">
import { useUserState } from "@/states/user-state";
import { publishMessage } from "@/utils/backend";
import { GroupId, GroupState, Message } from "@/utils/types";
import { VVirtualScroll } from "vuetify/labs/VVirtualScroll";
const props = defineProps<{
  groupId: GroupId | null;
  groupState: GroupState | null;
}>();

const { localPeerId, users } = storeToRefs(useUserState());

onMounted(() => {
  nextTick(() => {
    let scroll = document.getElementById("scroll");
    scroll?.scrollTo(0, scroll.scrollHeight);
  });
});

watch(
  () => props.groupState,
  () => {
    // console.log(scroll?.scrollTop, scroll?.scrollHeight);
    nextTick(() => {
      let scroll = document.getElementById("scroll");
      scroll?.scrollTo(0, scroll.scrollHeight);
    });
  },
  { deep: true }
);

function onSendText() {
  if (message.value == "") return;
  publishMessage(props.groupId!, { text: message.value });
  message.value = "";
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
