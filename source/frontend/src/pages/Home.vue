<script setup lang="ts">
import TitleBar from '../components/TitleBar.vue';
import TypescriptLogo from "../assets/TypeScript.png";
import RustLogo from "../assets/Rust.png";
import PgLogo from "../assets/PostgresSQL.png";
import VueLogo from "../assets/Vue.js.png";
import Docker from "../assets/Docker.png";
import GraphQlLogo from "../assets/GraphQL.png";
</script>

<template>
  <TitleBar />
  <div class="py-8 flex justify-center">
    <div class="w-[700px] flex flex-col gap-4 px-2">
      <p class="text-center font-bold">About this project</p>
      <p>
        This is an EDA (event-driven architecture) URL shortener written in Rust and TypeScript. By itself it is not a
        full project but more of playing around and learning new things, so you should never put this into a production
        environment.
      </p>

      <p class="text-center font-bold">Techstack</p>
      <div class="flex items-center justify-center gap-2">
        <img :src="RustLogo" class="h-12 w-fit" alt="Rust" title="Rust"></img>
        <img :src="TypescriptLogo" width="48px" height="48px" alt="Typescript" title="Typescript"></img>
        <img :src="VueLogo" width="48px" height="48px" alt="Vue" title="Vue"></img>
        <img :src="GraphQlLogo" width="48px" height="48px" alt="GraphQL" title="GraphQL"></img>
        <img :src="PgLogo" width="48px" height="48px" alt="PostgreSQL" title="PostgreSQL"></img>
        <img :src="Docker" width="48px" height="48px" alt="Docker" title="Docker"></img>
      </div>
    </div>
  </div>
</template>
