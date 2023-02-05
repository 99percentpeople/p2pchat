/// <reference types="vite/client" />

declare module "*.vue" {
  import type { DefineComponent } from "vue";
  const component: DefineComponent<{}, {}, any>;
  export default component;
}

import "vue-router";

declare module "vue-router" {
  interface RouteMeta {
    icon: string;
    title: string;
    bottom?: boolean;
  }
}
