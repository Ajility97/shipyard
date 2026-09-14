import { createRouter, createWebHashHistory } from "vue-router";
import EmptyView from "./views/EmptyView.vue";

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: "/", name: "home", component: EmptyView },
    { path: "/settings", name: "settings", component: EmptyView },
    { path: "/history", name: "history", component: EmptyView },
    { path: "/changelog", name: "changelog", component: EmptyView },
    { path: "/repo/:id", name: "repo", component: EmptyView },
  ],
});
