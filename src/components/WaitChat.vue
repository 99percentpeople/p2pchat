<template>
  <v-container
    v-if="status === 'noactive'"
    class="h-100 d-flex justify-center align-center"
  >
    <v-icon icon="mdi-message-outline" size="150" color="grey" />
  </v-container>
  <v-container
    v-if="status === 'nojoin'"
    class="h-100 d-flex flex-column justify-center align-center text-grey"
  >
    <v-icon icon="mdi-account-plus" size="150" />
    <v-btn variant="tonal" color="info" @click="$emit('join')">加入</v-btn>
  </v-container>
  <v-container
    v-if="status === 'noother'"
    class="h-100 d-flex flex-column justify-center align-center text-grey"
  >
    <v-icon icon="mdi-account-voice" size="150" />
    <span class="text-subtitle-1">等待其他用户</span>
  </v-container>
</template>

<script setup lang="ts">
defineProps<{
  status: "noactive" | "nojoin" | "noother";
}>();
const emit = defineEmits<{
  (e: "join"): void;
}>();
</script>
