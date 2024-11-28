<template>
  <NuxtLayout class="select-none w-screen h-screen">
    <NuxtPage />
  </NuxtLayout>
</template>

<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { AppStatus } from "~/types";
import { listen } from "@tauri-apps/api/event";
import { useAppState } from "./composables/app-state.js";
import { useRouter } from "#vue-router";

const router = useRouter();

const state = useAppState();
state.value = await invoke("fetch_state");

router.beforeEach(async () => {
  state.value = await invoke("fetch_state");
});

switch (state.value.status) {
  case AppStatus.NotConfigured:
    router.push({ path: "/setup" }).then(() => {
      console.log("Pushed Setup");
    });
    break;
  case AppStatus.SignedOut:
    router.push("/auth");
    break;
  case AppStatus.SignedInNeedsReauth:
    router.push("/auth/signedout");
    break;
  case AppStatus.ServerUnavailable:
    router.push("/error/serverunavailable");
    break;
  default:
    router.push("/store");
}

listen("auth/processing", () => {
  router.push("/auth/processing");
});

listen("auth/failed", () => {
  router.push("/auth/failed");
});

listen("auth/finished", () => {
  router.push("/store");
});

useHead({
  title: "Drop",
});
</script>
