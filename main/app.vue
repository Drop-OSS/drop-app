<template>
  <LoadingIndicator />
  <NuxtLayout class="select-none w-screen h-screen">
    <NuxtPage />
    <ModalStack />
  </NuxtLayout>
</template>

<script setup lang="ts">
import "~/composables/downloads.js";

import { invoke } from "@tauri-apps/api/core";
import { useAppState } from "./composables/app-state.js";
import {
  initialNavigation,
  setupHooks,
} from "./composables/state-navigation.js";

const router = useRouter();

const state = useAppState();

async function fetchState() {
  try {
    state.value = JSON.parse(await invoke("fetch_state"));
    if (!state.value)
      throw createError({
        statusCode: 500,
        statusMessage: `App state is: ${state.value}`,
        fatal: true,
      });
  } catch (e) {
    console.error("failed to parse state", e);
    throw e;
  }
}
await fetchState();

// This is inefficient but apparently we do it lol
router.beforeEach(async () => {
  await fetchState();
});

setupHooks();
initialNavigation(state);

useHead({
  title: "Drop",
});
</script>
