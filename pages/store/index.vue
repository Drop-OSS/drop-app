<template>
  <input placeholder="GAME ID" v-model="gameId" />
  <input placeholder="VERSION NAME" v-model="versionName" />

  <button
    class="w-full rounded-md p-4 bg-blue-600 text-white"
    @click="startGameDownload"
  >
    Download game
    <span v-if="progress != 0">
      ({{ Math.floor(progress * 1000) / 10 }}%)
    </span>
  </button>
  <button
    class="w-full rounded-md p-4 bg-blue-600 text-white"
    @click="stopGameDownload"
  >
    Cancel game download
  </button>
  <button
    class="w-full rounded-md p-4 bg-blue-600 text-white"
    @click="pause"
  >
    Pause game download
  </button>
  <button
    class="w-full rounded-md p-4 bg-blue-600 text-white"
    @click="resume"
  >
    Resume game download
  </button>

</template>
<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";

const gameId = ref("");
const versionName = ref("");
const progress = ref(0);

async function startGameDownload() {
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
async function stopGameDownload() {
  await invoke("cancel_game_download", { gameId: gameId.value });
}
async function pause() {
  await invoke("pause_game_downloads");
}
async function resume() {
  await invoke("resume_game_downloads");
}
</script>
