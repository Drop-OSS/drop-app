<template>
  <HeaderButton @click="() => close()">
    <XMarkIcon />
  </HeaderButton>
</template>

<script setup lang="ts">
import { MinusIcon, XMarkIcon } from "@heroicons/vue/16/solid";
import { getCurrentWindow } from "@tauri-apps/api/window";

async function close(){
  console.log(window);
  const result = await window.close();
  console.log(`closed window: ${result}`);
}

const window = getCurrentWindow();
</script>
