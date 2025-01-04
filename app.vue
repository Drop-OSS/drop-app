<template>
  <NuxtLayout class="select-none w-screen h-screen">
    <NuxtPage />
  </NuxtLayout>
  <ModalStack />
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
state.value = JSON.parse(await invoke("fetch_state"));

router.beforeEach(async () => {
  state.value = JSON.parse(await invoke("fetch_state"));
});

setupHooks();
initialNavigation(state);

listen("database_corrupted", (event) => {
  createModal(
      ModalType.Notification,
      {
        title: "Database corrupted",
        description: `Drop encountered an error while reading your download. A copy can be found at: "${(
          event.payload as unknown as string
        ).toString()}"`,
        buttonText: "Close"
      },
      (e, c) => c()
    );

})

useHead({
  title: "Drop",
});
</script>
