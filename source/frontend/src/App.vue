<script setup lang="ts">
import { onMounted } from 'vue';
import { useRouter, type RouteLocationNormalized } from 'vue-router';
import { getAuthProp } from './auth';

const router = useRouter();

onMounted(() => {
  router.beforeEach((to: RouteLocationNormalized) => {
    const auth = getAuthProp();

    if (to.meta.requiresAuth && (!auth || !auth.signedIn)) {
      return '/login';
    }

    return true;
  });
});
</script>

<template>
  <RouterView />
</template>
