<template>
  <NuxtLayout>
    <NuxtPage />
  </NuxtLayout>
  {{ state }}
</template>

<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
// @ts-expect-error
import { AppStatus } from "./types.d.ts";

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
</script>
