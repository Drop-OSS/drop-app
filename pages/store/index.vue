<template>

  <button class="w-full rounded-md p-4 bg-blue-600 text-white" @click="requestGameWrapper">
    Load Data
  </button>
  <button class="w-full rounded-md p-4 bg-blue-600 text-white" @click="requestGameWrapper">
    Download Game
  </button>
</template>
<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";

async function requestGame() {
  console.log("Requested game from FE");
  await invoke("start_game_download", { gameId: "328a276d-4777-4a47-97f1-15069c1e5f66", gameVersion: "1.11.2", maxThreads: 4 });
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
