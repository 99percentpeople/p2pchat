<template>
  <v-app full-height class="h-screen">
    <title-bar />
    <v-navigation-drawer
      permanent
      :rail="rail"
      v-model="drawer"
      color="primary"
    >
      <v-list nav>
        <v-list-item
          v-for="route in topRoutes"
          :key="route.name"
          :prepend-icon="route.meta?.icon"
          :title="title"
          :value="route.name"
          @click="$router.push({ name: route.name })"
          :active="route.name === $router.currentRoute.value.name"
        >
        </v-list-item>
      </v-list>

      <template #append>
        <v-divider />
        <v-list nav>
          <v-list-item
            v-for="route in bottomRoutes"
            :key="route.name"
            :prepend-icon="route.meta?.icon"
            :title="route.meta?.title"
            :value="route.name"
            @click="$router.push({ name: route.name })"
            :active="route.name === $router.currentRoute.value.name"
          >
          </v-list-item>
        </v-list>
      </template>
    </v-navigation-drawer>
    <v-app-bar :title="title" color="primary">
      <template #prepend>
        <v-app-bar-nav-icon
          variant="text"
          @click.stop="rail = !rail"
        ></v-app-bar-nav-icon>
      </template>
      <template v-slot:append>
        <v-btn
          v-for="action in actions"
          :key="action.name"
          :icon="action.icon"
          @click="action.action"
        ></v-btn>
      </template>
    </v-app-bar>
    <v-main class="h-100">
      <router-view>
        <template #="{ Component }">
          <keep-alive>
            <component
              :is="Component"
              @update-actions="onUpdateActions"
              @update-title="onUpdateTitle"
            />
          </keep-alive>
        </template>
      </router-view>
    </v-main>
  </v-app>
</template>
<script setup lang="ts">
import { listen } from "@tauri-apps/api/event";
import { onBeforeRouteUpdate } from "vue-router";

import {
  Group,
  GroupInfo,
  groups,
  listeners,
  startListen,
} from "./utils/backend";

const topRoutes = computed(() =>
  router.options.routes.filter((route) => !route.meta?.bottom)
);
const bottomRoutes = computed(() =>
  router.options.routes.filter((route) => route.meta?.bottom)
);
const route = useRoute();
const router = useRouter();
let title = $ref<string>(route.meta.title);
router.afterEach((to) => {
  title = to.meta.title;
});

let rail = $ref(true);
let drawer = $ref(true);

let actions = $ref<any>();

function onUpdateActions(acts: any) {
  actions = acts;
}
function onUpdateTitle(newTitle: string) {
  title = newTitle;
}
let unlisten: null | (() => void) = null;
const groupsList = reactive<{ [index: Group]: GroupInfo }>({});
onMounted(async () => {
  unlisten = await listen<string>("error", (event) => {
    console.error(
      `Got error in window ${event.windowLabel}, payload: ${event.payload}`
    );
  });
  if ((await listeners()).length === 0) {
    await startListen();
  }
  // clean groupsList
  for (let group in groupsList) {
    delete groupsList[group];
  }
  Object.assign(groupsList, await groups());
});

onUnmounted(() => {
  unlisten?.();
});
</script>

<style scoped lang="scss"></style>
