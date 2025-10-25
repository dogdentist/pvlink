<script setup lang="ts">
import { defineProps } from 'vue';

const props = defineProps({
  text: {
    type: String,
    required: true
  },
  call: {
    type: Function,
    required: true
  }
});

</script>

<template>
  <button @click="props.call()"
    class="border rounded-sm px-4 py-1.5 hover:cursor-pointer bg-black text-gray-50 hover:bg-gray-950 active:hover:bg-gray-900">{{
      props.text
    }}</button>
</template>