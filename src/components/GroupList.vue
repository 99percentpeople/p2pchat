<template>
  <v-list item-props lines="two">
    <v-list-subheader v-if="listName">{{ listName }}</v-list-subheader>
    <v-list-item
      v-for="(info, group) in items"
      :title="info.name"
      :value="group"
      :key="group"
      :active="group == active"
      @click="onSelect(group as string)"
    >
      <v-list-item-subtitle>
        {{ getGroupInfoMessage(info) }}
      </v-list-item-subtitle>
    </v-list-item>
  </v-list>
</template>
<script setup lang="ts">
import { Group, GroupInfo } from "../utils/backend";

const emit = defineEmits<{
  (e: "select", value: string): void;
}>();
function onSelect(group: Group) {
  active = group;
  emit("select", active);
}
const props = defineProps<{
  items: { [index: Group]: GroupInfo };
  listName?: string;
}>();
function getGroupInfoMessage(info: GroupInfo) {
  if (info.history.length === 0) {
    return "暂无消息";
  }
  const lastMessage = info.history[info.history.length - 1];
  if (lastMessage.message.text) {
    return lastMessage.message.text;
  } else if (lastMessage.message.file) {
    const file = lastMessage.message.file;
    return `文件 ${file.name}`;
  } else {
    return "未知消息";
  }
}
console.log(props.items);

let active = $ref<Group | null>(null);
</script>
