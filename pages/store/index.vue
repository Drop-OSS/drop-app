<template>
  <button
    class="w-full rounded-md p-4 bg-blue-600 text-white"
    @click="requestGameWrapper"
  >
    Load Data
  </button>
  <input placeholder="GAME ID" v-model="gameId" />
  <input placehodler="VERSION NAME" v-model="versionName" />
  <button
    class="w-full rounded-md p-4 bg-blue-600 text-white"
    @click="requestGameWrapper"
  >
    Download Game
  </button>
</template>
<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";

const gameId = ref("");
const versionName = ref("");

async function requestGame() {
  await invoke("start_game_download", {
    gameId: gameId.value,
    gameVersion: versionName.value,
    maxThreads: 4,
  });
}
function requestGameWrapper() {
  console.log("Wrapper started");
  requestGame()
    .then(() => {})
    .catch((e) => {
      console.log(e);
    });
}
</script>
