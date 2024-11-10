<template>
  <button
    class="w-full rounded-md p-4 bg-blue-600 text-white"
    @click="queueGameWrapper"
  >
    Queue Game Download
  </button>
  <input placeholder="GAME ID" v-model="gameId" />
  <input placeholder="VERSION NAME" v-model="versionName" />
  <input placeholder="STATUS" v-model="status" />
  <input placeholder="NEW ROOT DIR" v-model="newRootDir" />

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
    @click="setGameDownloadStatusWrapper"
  >
    Set game download progress
  </button>
  <button
    class="w-full rounded-md p-4 bg-blue-600 text-white"
    @click="changeRootDirectoryWrapper"
  >
    Change root download location
  </button>
  
</template>
<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";

const gameId = ref("");
const versionName = ref("");
const status = ref("");
const newRootDir = ref("");


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
  await invoke("start_game_downloads", { maxThreads: 4 });
  console.log("Finished downloading games");
}
function startGameDownloadsWrapper() {
  startGameDownloads()
    .then(() => {})
    .catch((e) => {
      console.log(e)
    })
}
function cancelGameDownloadWrapper() {
  console.log("Triggered game cancel wrapper");
  setGameDownloadStatus("Cancelled")
    .then(() => {})
    .catch((e) => {
      console.log(e)
    })
}
async function getGameDownloadProgress() {
  console.log("Getting game download status");
  await invoke("get_game_download_progress", { gameId: gameId.value });
}
function getGameDownloadProgressWrapper() {
  getGameDownloadProgress()
    .then(() => {})
    .catch((e) => {
      console.log(e)
    })
}
/* status can be any of the following values:
    Uninitialised,
    Queued,
    Paused,
    Manifest,
    Downloading,
    Finished,
    Stalled,
    Failed,
    Cancelled,
*/

async function setGameDownloadStatus(status: string) {
  console.log("Setting game download status");
  await invoke("set_download_state", { gameId: gameId.value, status: status });
}
function setGameDownloadStatusWrapper() {
  console.log("Called setGameDownloadWrapper");
  setGameDownloadStatus(status.value)
    .then(() => {})
    .catch((e) => {
      console.log(e)
    })
}
async function changeRootDirectory() {
  console.log("Changing root directory");
  await invoke("change_root_directory", { newDir: newRootDir.value });
}
function changeRootDirectoryWrapper() {
  changeRootDirectory()
    .then(() => {})
    .catch((e) => {
      console.log(e)
    })
}
</script>
