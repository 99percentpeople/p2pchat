<template>
  <v-layout full-height>
    <v-navigation-drawer permanent color="surface">
      <group-list
        v-if="joinedGroups[0].length !== 0"
        :items="joinedMap"
        list-name="已加入群组"
        @select="onGroupSelected"
      />
      <group-list
        v-if="joinedGroups[1].length !== 0"
        :items="notJoinedMap"
        list-name="未加入群组"
        @select="onGroupSelected"
      />
    </v-navigation-drawer>
    <v-main>
      <chat
        v-if="activedGroupStatus === 'active'"
        :local-peer-id="localPeer"
        :items="activedGroupMessages"
        @send="onSendMessage"
      />
      <wait-chat :status="activedGroupStatus" @join="onSubscribe" v-else>
      </wait-chat>
    </v-main>
  </v-layout>
  <v-dialog v-model="dialogVisible" persistent>
    <v-card>
      <v-card-title> 创建群组 </v-card-title>
      <v-card-text>
        <v-container>
          <v-text-field
            v-model="newGroupName"
            label="群组名称"
            outlined
          ></v-text-field>
        </v-container>
      </v-card-text>
      <v-card-actions>
        <v-spacer></v-spacer>
        <v-btn color="primary" @click="dialogVisible = false"> 取消 </v-btn>
        <v-btn color="primary" @click="onConfirm"> 创建 </v-btn>
      </v-card-actions>
    </v-card></v-dialog
  >
</template>

<script setup lang="ts">
import {
  Group,
  GroupInfo,
  GroupMessage,
  groups,
  localPeerId,
  Message,
  newGroup,
  publish,
  subscribe,
} from "../utils/backend";
import { listen } from "@tauri-apps/api/event";
const joinedGroups = computed(() => {
  let joined: Group[] = [];
  let notJoined: Group[] = [];
  console.log("joined groups", groupItems);

  Object.entries(groupItems).forEach(([group, info]) => {
    if (info.subscribers.find((id) => id === localPeer)) {
      joined.push(group);
    } else {
      notJoined.push(group);
    }
  });

  return [joined, notJoined];
});

const joinedMap = computed(() => {
  return joinedGroups.value[0].reduce(
    (obj, key) => ({ ...obj, [key]: groupItems[key] }),
    {}
  );
});

const notJoinedMap = computed(() => {
  return joinedGroups.value[1].reduce(
    (obj, key) => ({ ...obj, [key]: groupItems[key] }),
    {}
  );
});
let dialogVisible = $ref(false);
let activedGroup = $ref<Group | null>(null);
let groupItems = reactive<{ [index: Group]: GroupInfo }>({});

let activedGroupMessages = computed(() =>
  activedGroup ? groupItems[activedGroup].history : undefined
);
let newGroupName = $ref("");
const emit = defineEmits<{
  (
    e: "update-actions",
    actions: {
      name: string;
      icon: string;
      action: () => void;
    }[]
  ): void;
  (e: "update-title", title: string): void;
}>();
let localPeer = $ref("");
onActivated(() => {
  emit("update-actions", [
    {
      name: "Add",
      icon: "mdi-plus",
      action: () => {
        dialogVisible = true;
      },
    },
  ]);
});
onDeactivated(() => {
  emit("update-actions", []);
});

onMounted(async () => {
  localPeer = await localPeerId();
  await listen<{ [index: Group]: GroupInfo }>("group-update", (event) => {
    console.log("group update ", event.payload);
    for (let group in groupItems) {
      delete groupItems[group];
    }
    Object.assign(groupItems, event.payload);
  });
  await listen<[Group, GroupMessage]>("message", (event) => {
    console.log("message", event.payload);
    const [group, message] = event.payload;
    groupItems[group].history.push(message);
  });
  await onReloadGroups();
});
const activedGroupStatus = computed(() => {
  if (activedGroup) {
    if (
      joinedGroups.value[0].find((group) => group === activedGroup) ===
      undefined
    ) {
      return "nojoin";
    }

    if (groupItems[activedGroup].subscribers.length <= 1) {
      return "noother";
    }
    return "active";
  }

  return "noactive";
});

async function onSendMessage(message: Message) {
  if (activedGroup) {
    console.log("send message", message);
    const newGroupMessage = await publish(activedGroup, message);
    groupItems[activedGroup].history.push(newGroupMessage);
  }
}

function onGroupSelected(group: Group) {
  console.log("select group", group);
  activedGroup = group;
  emit("update-title", groupItems[group].name);
}

async function onReloadGroups() {
  for (let group in groupItems) {
    delete groupItems[group];
  }
  Object.assign(groupItems, await groups());
  console.log("reload groups", groupItems);
}

async function onConfirm() {
  dialogVisible = false;
  let [group, info] = await newGroup(newGroupName);
  groupItems[group] = info;
}

async function onSubscribe() {
  if (activedGroup) {
    let info = await subscribe(activedGroup);
    console.log(info);

    groupItems[activedGroup] = info;
  } else {
    console.log("no group selected");
  }
}
</script>

<style scoped lang="scss">
.select-list {
  border-right: 1px solid var(--el-border-color);
  display: flex;
  flex-direction: column;
}

.container {
  height: 100%;
}

.content {
  padding: 12px;
}

.group-list {
  flex: 1 1 auto;

  .group-list-item {
    width: 100%;
    padding: 12px;
    height: 40px;
    border-bottom: 1px solid var(--el-border-color);
    border-top: 1px solid var(--el-border-color);

    &:nth-child(2) {
      background-color: var(--el-border-color);
    }
  }

  .group-list-item::marker {
    content: none;
  }
}
</style>
