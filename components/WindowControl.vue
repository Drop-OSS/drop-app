<template>
  <HeaderButton v-if="showMinimise" @click="() => minimise()">
    <MinusIcon />
  </HeaderButton>
  <HeaderButton @click="() => close()">
    <XMarkIcon />
  </HeaderButton>
</template>

<script setup lang="ts">
import { MinusIcon, XMarkIcon } from "@heroicons/vue/16/solid";
import { getCurrentWindow } from "@tauri-apps/api/window";

const window = getCurrentWindow();
const showMinimise = await window.isMinimizable();

async function close() {
  await window.close();
}

async function minimise() {
  await window.minimize();
}
</script>
