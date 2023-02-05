import {
  createRouter,
  createWebHashHistory,
  RouteRecordRaw,
  RouterOptions,
} from "vue-router";

const routes: RouteRecordRaw[] = [
  {
    path: "/",
    name: "main",
    component: () => import("./pages/Main.vue"),
    meta: {
      icon: "mdi-home",
      title: "Home",
    },
  },
  {
    path: "/settings",
    name: "settings",
    component: defineAsyncComponent({
      loader: () => import("./pages/Settings.vue"),
    }),
    meta: {
      icon: "mdi-cog",
      title: "Setting",
      bottom: true,
    },
  },
];

const options: RouterOptions = {
  history: createWebHashHistory(),
  routes,
};

export default createRouter(options);
