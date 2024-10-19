<template>

  <button class="w-full rounded-md p-4 bg-blue-600 text-white" @click="requestGameWrapper">
    Load Data
  </button>
</template>
<script setup lang="ts">
definePageMeta({
  layout: "mini",
});

import { invoke } from "@tauri-apps/api/core";

async function requestGame() {
  console.log("Requested game from FE");
  await invoke("start_game_download", { gameId: "123", gameVersion: "1.2.3", maxThreads: 4 });
}
function requestGameWrapper() {
  console.log("Wrapper started");
  requestGame()
      .then(() => {})
      .catch((e) => {
        console.log(e);
      })
}
</script>