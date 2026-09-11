import { createRouter, createWebHashHistory } from "vue-router";
import EmptyView from "./views/EmptyView.vue";

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: "/", name: "home", component: EmptyView },
    { path: "/repo/:id", name: "repo", component: EmptyView },
  ],
});
