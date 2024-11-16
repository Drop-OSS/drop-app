<template>
  <input placeholder="GAME ID" v-model="gameId" />
  <input placeholder="VERSION NAME" v-model="versionName" />

  <button
    class="w-full rounded-md p-4 bg-blue-600 text-white"
    @click="startGameDownload"
  >
    Download game
    <span v-if="progress != 0"> ({{ Math.floor(progress * 1000) / 10 }}%) </span>
  </button>
</template>
<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";

const gameId = ref("");
const versionName = ref("");
const progress = ref(0);

async function startGameDownload() {
  await invoke("download_game", {
    gameId: gameId.value,
    gameVersion: versionName.value,
  });

  setInterval(() => {
    (async () => {
      const currentProgress = await invoke<number>(
        "get_current_game_download_progress",
        {
          gameId: gameId.value,
        }
      );
      console.log(currentProgress);
      progress.value = currentProgress;
    })();
  }, 100);
}
</script>
