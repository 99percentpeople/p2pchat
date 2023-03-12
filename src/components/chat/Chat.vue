<template>
  <chat-content
    v-if="status === 'active'"
    :group-state="selectedState"
    :group-id="groupId"
  />
  <v-container
    v-else-if="status === 'no-active'"
    class="h-100 d-flex justify-center align-center"
  >
    <v-icon icon="mdi-message-outline" size="150" color="grey" />
  </v-container>
  <v-container
    v-else-if="status === 'no-join'"
    class="h-100 d-flex flex-column justify-center align-center text-grey"
  >
    <v-icon icon="mdi-account-plus" size="150" />
    <v-btn variant="tonal" color="info" @click="onJoin">加入</v-btn>
  </v-container>
  <v-container
    v-else-if="status === 'no-other'"
    class="h-100 d-flex flex-column justify-center align-center text-grey"
  >
    <v-icon icon="mdi-account-voice" size="150" />
    <span class="text-subtitle-1">等待其他用户</span>
  </v-container>
</template>
<script setup lang="ts">
import { subscribe } from "@/utils/backend";
import { GroupId } from "@/utils/types";
import { useGroupState } from "@/states/group-state";
import { useUserState } from "@/states/user-state";
const props = defineProps<{
  groupId: GroupId | null;
}>();

const { groupStates } = storeToRefs(useGroupState());
const { localPeerId } = storeToRefs(useUserState());
const selectedState = computed(() => {
  return props.groupId ? groupStates.value[props.groupId] : null;
});
const status = computed(() => {
  if (!selectedState.value || !localPeerId.value) {
    return "no-active";
  }
  if (!selectedState.value.subscribers.includes(localPeerId.value)) {
    return "no-join";
  }
  if (selectedState.value.subscribers.length < 2) {
    return "no-other";
  }
  return "active";
});

function onJoin() {
  if (props.groupId) {
    subscribe(props.groupId);
  }
}
</script>
<style scoped lang="scss"></style>
