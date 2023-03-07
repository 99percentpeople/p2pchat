<template>
  <span v-if="msg.text">
    {{ msg.text }}
  </span>
  <span v-else-if="msg.file"> </span>
</template>

<script setup lang="ts">
import { Message } from "@/utils/types";

const props = defineProps<{
  msg: Message;
}>();
</script>
