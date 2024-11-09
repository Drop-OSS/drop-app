<template>
  <button
    class="w-full rounded-md p-4 bg-blue-600 text-white"
    @click="queueGameWrapper"
  >
    Queue Game Download
  </button>
  <input placeholder="GAME ID" v-model="gameId" />
  <input placeholder="VERSION NAME" v-model="versionName" />
  <button
    class="w-full rounded-md p-4 bg-blue-600 text-white"
    @click="startGameDownloadsWrapper"
  >
    Start Game Downloads
  </button>
  <button
    class="w-full rounded-md p-4 bg-blue-600 text-white"
    @click="cancelGameDownloadWrapper"
  >
    Cancel game download
  </button>
  <button
    class="w-full rounded-md p-4 bg-blue-600 text-white"
    @click="getGameDownloadProgressWrapper"
  >
    Get game download progress
  </button>
  <button
    class="w-full rounded-md p-4 bg-blue-600 text-white"
    @click="pauseGameDownloadWrapper"
  >
    Pause game download
  </button>
  <button
    class="w-full rounded-md p-4 bg-blue-600 text-white"
    @click="resumeGameDownloadWrapper"
  >
    Resume game download
  </button>
</template>
<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";

const gameId = ref("");
const versionName = ref("");

async function queueGame() {
  await invoke("queue_game_download", {
    gameId: gameId.value,
    gameVersion: versionName.value,
    maxThreads: 12,
  });
  console.log("Requested game from FE");
}
function queueGameWrapper() {
  console.log("Wrapper started");
  queueGame()
    .then(() => {})
    .catch((e) => {
      console.log(e);
    });
}
async function startGameDownloads() {
  console.log("Downloading Games");
  await invoke("start_game_downloads", { maxThreads: 4 })
  console.log("Finished downloading games");
}
function startGameDownloadsWrapper() {
  startGameDownloads()
    .then(() => {})
    .catch((e) => {
      console.log(e)
    })
}
async function cancelGameDownload() {
  console.log("Cancelling game download");
  await invoke("cancel_specific_game_download", { gameId: gameId.value })
}
function cancelGameDownloadWrapper() {
  console.log("Triggered game cancel wrapper");
  cancelGameDownload()
    .then(() => {})
    .catch((e) => {
      console.log(e)
    })
}
async function getGameDownloadProgress() {
  console.log("Getting game download status");
  await invoke("get_game_download_progress", { gameId: gameId.value })
}
function getGameDownloadProgressWrapper() {
  getGameDownloadProgress()
    .then(() => {})
    .catch((e) => {
      console.log(e)
    })

}
async function pauseGameDownload() {
  console.log("Getting game download status");
  await invoke("pause_game_download", { gameId: gameId.value })
}
function pauseGameDownloadWrapper() {
  pauseGameDownload()
    .then(() => {})
    .catch((e) => {
      console.log(e)
    })

}
async function resumeGameDownload() {
  console.log("Getting game download status");
  await invoke("resume_game_download", { gameId: gameId.value })
}
function resumeGameDownloadWrapper() {
  resumeGameDownload()
    .then(() => {})
    .catch((e) => {
      console.log(e)
    })

}
</script>
