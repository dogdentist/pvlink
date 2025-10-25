import { createApp } from "vue";
import "./style.css";
import App from "./App.vue";
import HomeView from './pages/Home.vue'
import LoginView from './pages/Login.vue'
import { createRouter, createWebHistory } from "vue-router";
import DashboardView from "./pages/Dashboard.vue";
import LinkView from "./pages/Link.vue";
import AddLink from "./pages/AddLink.vue";

const routes = [
  { path: '/', component: HomeView },
  { path: '/login', component: LoginView },
  {
    path: '/dashboard',
    component: DashboardView,
    meta: { requiresAuth: true }
  },
  {
    path: '/dashboard/link/:id',
    component: LinkView,
    meta: { requiresAuth: true },
  },
  {
    path: '/dashboard/add',
    component: AddLink,
    meta: { requiresAuth: true },
  },
]

export const router = createRouter({
  history: createWebHistory(),
  routes,
})

createApp(App).use(router).mount("#app");
