<template>
  <HeaderButton @click="() => window.close()">
    <XMarkIcon />
  </HeaderButton>
</template>

<script setup lang="ts">
import { MinusIcon, XMarkIcon } from "@heroicons/vue/16/solid";
import { getCurrentWindow } from "@tauri-apps/api/window";

const window = getCurrentWindow();
</script>
