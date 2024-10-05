<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

const greetMsg = ref("");
const name = ref("");

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
  const start = Date.now();
  await invoke("unpack_debug");
  const ms = Date.now() - start;
  greetMsg.value = `Took ${ms}ms`;
}
</script>

<template>
  <form class="row" @submit.prevent="greet">
    <input id="greet-input" v-model="name" placeholder="Enter a name..." />
    <button type="submit">Unpack</button>
  </form>

  <p>{{ greetMsg }}</p>
</template>
