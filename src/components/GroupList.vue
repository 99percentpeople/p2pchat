<template>
  <v-list item-props lines="two" v-if="Object.keys(groups).length !== 0">
    <!-- <v-list-subheader>已加入</v-list-subheader> -->
    <v-list-item
      v-for="(info, group) in groups"
      :title="info.name"
      :value="group"
      :key="group"
      :active="group === active"
      @click="onSelect(group as string)"
    >
      <v-list-item-subtitle>
        {{ getLastMessage(group as string) }}
      </v-list-item-subtitle>
    </v-list-item>
  </v-list>
  <!-- <v-list
    item-props
    lines="two"
    v-if="groupsNotJoined.state.value.length !== 0"
  >
    <v-list-subheader>未加入</v-list-subheader>
    <v-list-item
      v-for="group in groupsNotJoined.state.value"
      :title="groups.state.value[group].name"
      :value="group"
      :key="group"
      :active="group === active"
      @click="onSelect(group as string)"
    >
      <v-list-item-subtitle>
        {{ lastMessage[group] }}
      </v-list-item-subtitle>
    </v-list-item>
  </v-list> -->
  <v-container
    v-else
    class="h-100 d-flex flex-column justify-center align-center text-grey"
  >
    <v-icon icon="mdi-account-group" size="80"></v-icon>
    暂无群组
  </v-container>
</template>
<script setup lang="ts">
import { GroupId } from "@/utils/types";
import { useGroupState } from "@/states/group-state";
const emit = defineEmits<{
  (e: "select", value: GroupId): void;
}>();
const active = ref<GroupId | null>(null);

function onSelect(group: GroupId) {
  active.value = group;
  emit("select", active.value);
}

const { groups, groupStates } = storeToRefs(useGroupState());

function getLastMessage(groupId: GroupId): string {
  let status = groupStates.value[groupId];
  if (!status) {
    return "加载中";
  }
  if (status.history.length === 0) {
    return "暂无消息";
  }
  const lastMessage = status.history[status.history.length - 1];
  if (lastMessage.message.text) {
    return lastMessage.message.text;
  } else if (lastMessage.message.file) {
    const file = lastMessage.message.file;
    return `文件 ${file.name}`;
  } else {
    return "未知消息";
  }
}
</script>
