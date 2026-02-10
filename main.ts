import { createApp } from "vue";
import { createRouter, createWebHashHistory } from "vue-router";
import App from "./App.vue";
import "./style.css";

// 导入视图组件
import DesignStudio from "./views/DesignStudio.vue";
import CodeHubAssistant from "./views/CodeHubAssistant.vue";
import MetricsDashboard from "./views/MetricsDashboard.vue";

const routes = [
  { path: "/", redirect: "/design" },
  { path: "/design", component: DesignStudio },
  { path: "/codehub", component: CodeHubAssistant },
  { path: "/metrics", component: MetricsDashboard },
];

const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

const app = createApp(App);
app.use(router);
app.mount("#app");
