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
  },
});

createApp(App).use(vuetify).use(router).use(createPinia()).mount("#app");
