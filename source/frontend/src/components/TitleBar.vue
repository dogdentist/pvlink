<script setup lang="ts">
import { useRouter } from "vue-router";
import LogoSvg from "../assets/logo.svg"
import { getAuthProp } from "../auth";

const router = useRouter();
const authProp = getAuthProp();
</script>

<template>
  <div class="fixed top-0 left-0 w-full h-[50px] px-4 py-2 flex items-center z-50 bg-black">
    <button @click="router.push('/')" class="w-fit flex flex-row px-2 hover:cursor-pointer gap-2 text-gray-50">
      <img :src='LogoSvg'></img>
      <div class="flex items-center">
        Home
      </div>
    </button>
    <div class="flex-auto"></div>
    <button v-if="authProp?.signedIn ?? false" @click="router.push('/dashboard')"
      class="w-fit px-2 hover:cursor-pointer flex items-center rounded-full p-1 bg-gray-100 text-black">
      Dashboard
    </button>
    <button v-else @click="router.push('/login')"
      class="w-fit px-2 hover:cursor-pointer flex items-center rounded-full p-1 bg-gray-100 text-black">
      Sign-in
    </button>
  </div>
  <div class="mb-[50px]"></div>
</template>