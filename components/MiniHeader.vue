<template>
  <div
    @mousedown="() => window.startDragging()"
    class="cursor-pointer flex flex-row items-center justify-between bg-zinc-950 px-3 py-2"
  >
    <div>
      <Wordmark />
    </div>
    <WindowControl />
  </div>
</template>

<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";

const window = getCurrentWindow();
</script>
