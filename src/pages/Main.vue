<template>
  <v-layout full-height>
    <v-navigation-drawer permanent color="surface">
      <group-list @select="onGroupSelected" />
    </v-navigation-drawer>
    <v-main>
      <chat :group-id="activedGroup" />
    </v-main>
  </v-layout>
  <v-dialog v-model="newGroupDialogVisible" persistent>
    <v-card>
      <v-card-title> 创建群组 </v-card-title>
      <v-card-text>
        <v-container>
          <v-row>
            <v-col cols="12" md="6">
              <v-text-field
                v-model="newGroupInfo.name"
                label="群组名称"
                outlined
              ></v-text-field>
            </v-col>
            <v-col cols="12" md="6">
              <v-text-field
                v-model="newGroupInfo.description"
                label="群组描述"
                outlined
              ></v-text-field>
            </v-col>
          </v-row>
        </v-container>
      </v-card-text>
      <v-card-actions>
        <v-spacer></v-spacer>
        <v-btn color="primary" @click="newGroupDialogVisible = false">
          取消
        </v-btn>
        <v-btn color="primary" @click="onNewGroup"> 创建 </v-btn>
      </v-card-actions>
    </v-card></v-dialog
  >
  <v-dialog v-model="groupInfoDialogVisible">
    <v-card>
      <v-card-title> 群组信息 </v-card-title>
      <v-card-text>
        <v-list>
          <v-list-item>
            <v-list-item-title>群组ID</v-list-item-title>
            <v-list-item-subtitle>
              {{ activedGroup }}
            </v-list-item-subtitle>
          </v-list-item>
          <v-list-item>
            <v-list-item-title>名称</v-list-item-title>
            <v-list-item-subtitle>
              {{ groupItems.state.value[activedGroup!].name }}
            </v-list-item-subtitle>
          </v-list-item>
          <v-list-item v-if="groupItems.state.value[activedGroup!].description">
            <v-list-item-title>描述</v-list-item-title>
            <v-list-item-subtitle>
              {{ groupItems.state.value[activedGroup!].description }}
            </v-list-item-subtitle>
          </v-list-item>
        </v-list>
      </v-card-text>
      <v-card-actions>
        <v-spacer></v-spacer>
        <v-btn color="primary" @click="groupInfoDialogVisible = false">
          返回
        </v-btn>
      </v-card-actions>
    </v-card></v-dialog
  >
</template>

<script setup lang="ts">
import { getGroups, newGroup } from "../utils/backend";
import { GroupId, GroupInfo } from "../utils/types";
import { listen } from "@tauri-apps/api/event";
import { Action } from "../utils/types";
const emit = defineEmits<{
  (e: "update-actions", actions: Action[]): void;
  (e: "update-title", title: string): void;
}>();
let newGroupDialogVisible = ref(false);
let groupInfoDialogVisible = ref(false);
let activedGroup = ref<GroupId | null>(null);
let groupItems = useAsyncState(async () => {
  return await getGroups();
}, {});
let newGroupInfo = reactive<GroupInfo>({
  name: "",
  description: null,
});
let actions = computed<Action[]>(() => {
  let acts = [
    {
      name: "Add",
      icon: "mdi-plus",
      action: () => {
        newGroupDialogVisible.value = true;
      },
    },
  ];
  if (activedGroup.value) {
    acts.push({
      name: "Info",
      icon: "mdi-information",
      action: () => {
        groupInfoDialogVisible.value = true;
      },
    });
  }
  return acts;
});
watch(
  () => actions.value,
  (newActions) => {
    emit("update-actions", newActions);
  },
  { deep: true }
);
onMounted(() => {
  listen<[GroupId, GroupInfo]>("group-update", async () => {
    await groupItems.execute();
  });
});

onActivated(() => {
  emit("update-actions", actions.value);
});
onDeactivated(() => {
  emit("update-actions", []);
});

function onGroupSelected(active: GroupId) {
  activedGroup.value = active;
  if (activedGroup.value) {
    emit("update-title", groupItems.state.value[activedGroup.value].name);
  }
}
async function onNewGroup() {
  if (newGroupInfo.name === "") return;
  await newGroup(newGroupInfo);
  newGroupDialogVisible.value = false;
  newGroupInfo.name = "";
  newGroupInfo.description = null;
}
</script>

<style scoped lang="scss"></style>
