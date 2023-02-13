import { createApp } from "vue";
import App from "./App.vue";
import router from "./router";
import { createPinia } from "pinia";
import { createVuetify } from "vuetify";
import "@mdi/font/css/materialdesignicons.css";
import "vuetify/styles";
import "./styles/main.scss";

const vuetify = createVuetify({
  theme: {
    defaultTheme: "light",
    variations: {
      colors: ["primary", "secondary"],
      lighten: 2,
      darken: 2,
    },
    themes: {},
  },
});

createApp(App).use(vuetify).use(router).use(createPinia()).mount("#app");
