import { createApp } from "vue";
import { createRouter, createWebHashHistory } from "vue-router";
import App from "./App.vue";
import "./style.css";

import DesignStudio from "./views/DesignStudio.vue";

const routes = [
  { path: "/", redirect: "/editor" },
  { path: "/editor", component: DesignStudio },
];

const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

const app = createApp(App);
app.use(router);
app.mount("#app");
