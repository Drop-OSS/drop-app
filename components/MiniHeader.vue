<template>
  <div
    @mousedown="() => window.startDragging()"
    class="h-10 cursor-pointer flex flex-row items-center justify-between bg-zinc-950"
  >
    <div class="px-5 py-3">
      <Wordmark class="mt-1" />
    </div>
    <WindowControl />
  </div>
</template>

<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";

const window = getCurrentWindow();
</script>
