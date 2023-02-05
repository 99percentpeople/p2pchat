<template>
  <div class="chat-layout">
    <v-virtual-scroll
      v-if="items"
      :items="items"
      item-key="timestamp"
      height="0"
      class="px-2"
      id="scroll"
    >
      <template #default="{ item, index }">
        <chat-message :self="localPeerId == item.source">
          <chat-content :msg="item.message" />
        </chat-message>
      </template>
    </v-virtual-scroll>
    <v-card v-else class="flex-grow-1"> </v-card>
    <v-toolbar :elevation="8" class="px-4" color="background" v-if="items">
      <v-text-field
        variant="underlined"
        v-model="message"
        append-icon="mdi-send"
        @click:append="onSendText"
      >
      </v-text-field>
    </v-toolbar>
  </div>
</template>
<script setup lang="ts">
import { VVirtualScroll } from "vuetify/labs/VVirtualScroll";
import { GroupMessage, Message } from "../../utils/backend";

let emit = defineEmits<{
  (e: "send", message: Message): void;
}>();

const props = defineProps<{
  localPeerId: string;
  items?: GroupMessage[];
}>();
onMounted(async () => {
  let scroll = document.getElementById("scroll");
  scroll?.scrollTo(0, scroll.scrollHeight);
});
watch(
  () => props.items,
  async () => {
    let scroll = document.getElementById("scroll");
    // scroll?.scrollTo(0, 0);
    await nextTick();
    console.log(scroll?.clientHeight);
    scroll?.scrollTo(0, scroll.scrollHeight);
  },
  { deep: true, immediate: true }
);

function onSendText() {
  if (message == "") return;
  emit("send", { text: message });
  message = "";
}

let message = $ref("");
</script>
<style scoped lang="scss">
.chat-layout {
  display: flex;
  flex-direction: column;
  height: 100%;
}
</style>
