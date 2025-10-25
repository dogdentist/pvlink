<script setup lang="ts">
import { computed, onMounted, ref, type Ref } from 'vue';
import TitleBar from '../components/TitleBar.vue'
import VueDatePicker from '@vuepic/vue-datepicker';
import '@vuepic/vue-datepicker/dist/main.css'
import Button from '../components/Button.vue';
import { ApolloClient, gql } from '@apollo/client';
import { HttpLink } from '@apollo/client';
import { InMemoryCache } from '@apollo/client';
import { ServerError } from '@apollo/client';
import { setAuthProp } from '../auth';
import { useRouter } from 'vue-router';

const lut = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
const shortLinkLength: Ref<number> = ref(16);
const fieldShortLink: Ref<string | null> = ref(null);
const fieldTargetLink: Ref<string | null> = ref(null);
const fieldExpirationDate: Ref<string | null> = ref(null);
const errorMessage: Ref<string | null> = ref(null);
const router = useRouter();

const minDate = computed(() => {
  const tomorrow = new Date();

  tomorrow.setDate(tomorrow.getDate() + 1);

  return tomorrow.toISOString().split('T')[0];
});

const client = new ApolloClient({
  link: new HttpLink({ uri: "/api" }),
  cache: new InMemoryCache(),
});

const generateShortLink = () => {
  let v = '';

  for (let i = 0; i < shortLinkLength.value; i++) {
    v += lut.charAt(Math.floor(Math.random() * lut.length));
  }

  fieldShortLink.value = v;
};

const addLink = async () => {
  if (!fieldShortLink.value || !fieldTargetLink.value) {
    errorMessage.value = "Empty fields."
    return;
  };

  const shortLink = fieldShortLink.value.trim();

  if (!shortLink.split('').every(char => lut.includes(char))) {
    errorMessage.value = "The short link must contain the only charaters: a-z, A-Z, or 0-9."
    return;
  }

  try {
    const res = await client.mutate({
      mutation: gql`
        mutation createLink($shortLink: String!, $targetLink: String!, $expiration: Int) {
          createLink(shortLink: $shortLink, targetLink: $targetLink, expiration: $expiration)
        }
      `,
      variables: {
        shortLink: shortLink,
        targetLink: fieldTargetLink.value.trim(),
        expiration: fieldExpirationDate.value ? Math.floor(new Date(fieldExpirationDate.value).getTime() / 1000) : null
      }
    });

    if ((res.data as { createLink: boolean }).createLink) {
      router.push("/dashboard");
    } else {
      errorMessage.value = "Short link already exist."
    }
  } catch (err: any) {
    if (ServerError.is(err)) {
      const serverError = err as ServerError;

      if (serverError.statusCode == 401) {
        setAuthProp(null);
        router.push("/login");
      }
    }

    console.error(`api call failed: ${err}`);
  }
};

onMounted(() => {
  generateShortLink();
});
</script>

<template>
  <TitleBar />
  <div class="w-full py-[2%] flex justify-center">
    <div class="w-[350px] flex flex-col gap-4">
      <div v-if="errorMessage?.length ?? 0 > 0" class="text-red-300!">
        {{ errorMessage }}
      </div>
      <div class="flex flex-col gap-0.5">
        <p>Short link</p>
        <input v-model="fieldShortLink" type="text" autocapitalize="false" spellcheck="false"
          class="px-2 py-1 rounded-sm"></input>
      </div>
      <div class="flex flex-col gap-0.5">
        <p>Redirect link</p>
        <input v-model="fieldTargetLink" type="text" autocapitalize="false" spellcheck="false"
          class="px-2 py-1 rounded-sm"></input>
      </div>
      <div class="flex flex-col gap-0.5">
        <p>Expiration*:</p>
        <VueDatePicker v-model="fieldExpirationDate" :min-date="minDate" format="MM/dd/yyyy" dark text-input
          class="text-white date-picker-dark" />
      </div>
      <div class="flex justify-center">
        <Button :text="'Create'" :call="addLink" class="w-[90px]" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.dp__theme_dark {
  --dp-background-color: #000000;
  --dp-text-color: #fff;
  --dp-hover-color: #484848;
  --dp-hover-text-color: #fff;
  --dp-hover-icon-color: #959595;
  --dp-primary-color: #005cb2;
  --dp-primary-disabled-color: #61a8ea;
  --dp-primary-text-color: #fff;
  --dp-secondary-color: #a9a9a9;
  --dp-border-color: #fff;
  --dp-menu-border-color: #2d2d2d;
  --dp-border-color-hover: #fff;
  --dp-border-color-focus: #fff;
  --dp-disabled-color: #737373;
  --dp-disabled-color-text: #d0d0d0;
  --dp-scroll-bar-background: #212121;
  --dp-scroll-bar-color: #484848;
  --dp-success-color: #00701a;
  --dp-success-color-disabled: #428f59;
  --dp-icon-color: #959595;
  --dp-danger-color: #e53935;
  --dp-marker-color: #e53935;
  --dp-tooltip-color: #3e3e3e;
  --dp-highlight-color: rgb(0 92 178 / 20%);
  --dp-range-between-dates-background-color: var(--dp-hover-color, #484848);
  --dp-range-between-dates-text-color: var(--dp-hover-text-color, #fff);
  --dp-range-between-border-color: var(--dp-hover-color, #fff);
}
</style>
