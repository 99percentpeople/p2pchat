<template>
  <div class="chat-row my-4" :class="self ? 'chat-row-self' : 'chat-row'">
    <v-avatar size="40" v-if="!self" :image="userInfo.avatar ?? defaultAvatar">
    </v-avatar>
    <div
      class="rounded pa-1 text-white d-flex align-center justify-center text-body-1"
      :class="self ? 'chat-bubble-self' : 'chat-bubble'"
    >
      <slot></slot>
    </div>
    <v-avatar size="40" v-if="self" :image="userInfo.avatar ?? defaultAvatar">
    </v-avatar>
  </div>
</template>
<script setup lang="ts">
import { PeerId, UserInfo } from "@/utils/types";
import defaultAvatar from "/avatar.webp";
import { useUserState } from "@/states/user-state";
const { localPeerId } = storeToRefs(useUserState());
const props = defineProps<{
  peerId: PeerId;
  userInfo: UserInfo;
}>();
const self = computed(() => {
  return props.peerId === localPeerId.value;
});
</script>
<style scoped lang="scss">
@use "@/styles/settings.scss";
.chat-bubble {
  max-width: 60%;
  position: relative;
  word-break: break-all;
  left: 8px;
  z-index: 1;
  background-color: map-get(settings.$blue, lighten-2);
  border: 1px solid map-get(settings.$blue, lighten-2);
  &::before {
    content: "";
    width: 8px;
    height: 8px;
    position: absolute;
    top: 16px;
    left: -4px;
    transform: rotate(45deg);
    background-color: map-get(settings.$blue, lighten-2);
    border: 1px solid map-get(settings.$blue, lighten-2);
    border-style: none none solid solid;
    z-index: -999;
  }
}
.chat-bubble-self {
  max-width: 60%;
  position: relative;
  word-break: break-all;
  right: 8px;
  z-index: 1;
  background-color: map-get(settings.$green, lighten-2);
  border: 1px solid map-get(settings.$green, lighten-2);
  &::before {
    content: "";
    width: 8px;
    height: 8px;
    position: absolute;
    top: 16px;
    right: -4px;
    transform: rotate(45deg);
    background-color: map-get(settings.$green, lighten-2);
    border: 1px solid map-get(settings.$green, lighten-2);
    border-style: solid solid none none;
    z-index: -999;
  }
}
.chat-row {
  display: flex;
  position: relative;
  flex-direction: row;
  justify-content: flex-start;
  margin-bottom: 8px;
}
.chat-row-self {
  display: flex;
  position: relative;
  flex-direction: row;
  justify-content: flex-end;
  margin-bottom: 8px;
}
</style>
