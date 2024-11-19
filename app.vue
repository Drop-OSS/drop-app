<template>
  <NuxtLayout class="select-none w-screen h-screen">
    <NuxtPage />
  </NuxtLayout>
</template>

<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { useAppState } from "./composables/app-state.js";
import { useRouter } from "#vue-router";
import {
  initialNavigation,
  setupHooks,
} from "./composables/state-navigation.js";

const router = useRouter();

const state = useAppState();
state.value = await invoke("fetch_state");

router.beforeEach(async () => {
  state.value = await invoke("fetch_state");
});

setupHooks();
initialNavigation(state);

useHead({
  title: "Drop",
});
</script>
