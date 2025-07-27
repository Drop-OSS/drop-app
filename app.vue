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
import { AppStatus } from "~/types";
import { listen } from "@tauri-apps/api/event";
import { useAppState } from "./composables/app-state.js";
import {
  initialNavigation,
  setupHooks,
} from "./composables/state-navigation.js";

const router = useRouter();

const state = useAppState();
try {
  state.value = JSON.parse(await invoke("fetch_state"));
} catch (e) {
  console.error("failed to parse state", e);
}

router.beforeEach(async () => {
  try {
    state.value = JSON.parse(await invoke("fetch_state"));
  } catch (e) {
    console.error("failed to parse state", e);
  }
});

setupHooks();
initialNavigation(state);

useHead({
  title: "Drop",
});
</script>
