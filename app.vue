<template>
  <div class="flex flex-col bg-zinc-900 h-screen w-screen overflow-hidden">
    <Header class="select-none" />
    <div class="grow overflow-y-scroll">
        <div class="h-[200vh]" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";

const state = await invoke("fetch_state");
</script>
