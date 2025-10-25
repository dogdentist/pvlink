<script setup lang="ts">
import ky from "ky";
import TitleBar from '../components/TitleBar.vue';
import Button from '../components/Button.vue';
import { type Ref, ref } from "vue";
import { useRouter } from "vue-router";
import { setAuthProp } from "../auth";

const errorMessage: Ref<string | null> = ref(null);
const fieldUsername: Ref<string> = ref("");
const fieldPassword: Ref<string> = ref("");
const router = useRouter();

const loginClick = async () => {
  try {
    const res = await ky.post("/api/auth/login", {
      json: {
        username: fieldUsername.value,
        password: fieldPassword.value
      },
      throwHttpErrors: false
    });

    const username = fieldUsername.value;

    fieldUsername.value = "";
    fieldPassword.value = "";

    if (res.status == 200) {
      errorMessage.value = null;

      setAuthProp({
        signedIn: true,
        username: username
      });

      router.push("/");
    } else {
      errorMessage.value = "Bad username or bad password"
    }
  } catch (err) {
    errorMessage.value = "Error occured while signing you in"
  }
};
</script>

<template>
  <TitleBar />
  <div class="w-full py-[2%] flex justify-center">
    <div class="w-[350px] flex flex-col gap-4">
      <span class="text-base">Try the username <span class="text-blue-500!">admin</span> and password <span
          class="text-blue-500!">admin</span></span>
      <div class="flex flex-col gap-2">
        <div v-if="errorMessage?.length ?? 0 > 0" class="text-red-300! text-center">
          {{ errorMessage }}
        </div>
        <div class="flex flex-col gap-0.5">
          <p>Username</p>
          <input v-model="fieldUsername" @keyup.enter="loginClick" type="text" autocapitalize="false" spellcheck="false"
            class="px-2 py-1 rounded-sm"></input>
        </div>
        <div class="flex flex-col gap-0.5">
          <p>Password</p>
          <input v-model="fieldPassword" @keyup.enter="loginClick" type="password" class="px-2 py-1 rounded-sm"></input>
        </div>
      </div>
      <div class="flex justify-center">
        <Button :text="'Sign-in'" :call="loginClick" class="w-[90px]" />
      </div>
    </div>
  </div>
</template>
