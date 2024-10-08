<template>
  <NuxtLayout class="select-none">
    <NuxtPage />
  </NuxtLayout>
  {{ state }}
  <input type="text" v-model="debugLocation" />
  <NuxtLink :to="debugLocation">Go</NuxtLink>
</template>

<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
// @ts-expect-error
import { AppStatus } from "./types.d.ts";
import { listen } from "@tauri-apps/api/event";

const router = useRouter();

const state: { status: AppStatus } = await invoke("fetch_state");
switch (state.status) {
  case AppStatus.NotConfigured:
    router.push("/setup");
    break;
  case AppStatus.SignedOut:
    router.push("/auth");
    break;
}

listen("auth/processing", () => {
  router.push("/auth/processing");
});

listen("auth/failed", () => {
  router.push("/auth/failed");
});

listen("auth/finished", () => {
  router.push("/");
});

const debugLocation = ref("");
</script>
