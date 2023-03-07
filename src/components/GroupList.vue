<template>
  <v-list
    item-props
    lines="two"
    v-if="Object.keys(groups.state.value).length !== 0"
  >
    <!-- <v-list-subheader>已加入</v-list-subheader> -->
    <v-list-item
      v-for="(info, group) in groups.state.value"
      :title="info.name"
      :value="group"
      :key="group"
      :active="group === active"
      @click="onSelect(group as string)"
    >
      <v-list-item-subtitle>
        {{ lastMessage[group] }}
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
import { listen } from "@tauri-apps/api/event";
import { computedAsync, useAsyncState } from "@vueuse/core";
import { getGroupStatus, localPeerId, getGroups } from "@/utils/backend";
import { GroupId, GroupInfo } from "@/utils/types";
const emit = defineEmits<{
  (e: "select", value: GroupId): void;
}>();
const active = ref<GroupId | null>(null);

function onSelect(group: GroupId) {
  active.value = group;
  emit("select", active.value);
}
onBeforeMount(() => {
  localPeerId().then((id) => {
    localPeer.value = id;
  });
});
onMounted(() => {
  listen<[GroupId, GroupInfo]>("group-update", async (event) => {
    let [group, info] = event.payload;
    let newStatus = Object.defineProperty(
      {
        ...groups.state.value,
      },
      group,
      {
        value: info,
        enumerable: true,
      }
    );
    groups.state.value = newStatus;
  });
});
let localPeer = ref("");

let groups = useAsyncState(async () => {
  return await getGroups();
}, {} as { [index: GroupId]: GroupInfo });

let lastMessage = computedAsync<{ [index: GroupId]: string }>(async () => {
  let result: { [index: string]: string } = {};
  for (const group in groups.state.value) {
    result[group] = await getLastMessage(group);
  }
  return result;
});

async function getLastMessage(groupId: GroupId): Promise<string> {
  let status = await getGroupStatus(groupId);

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
