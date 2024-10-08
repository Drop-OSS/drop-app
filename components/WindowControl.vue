<template>
  <div class="inline-flex items-center gap-x-2">
    <HeaderButton @click="() => window.minimize()">
      <MinusIcon />
    </HeaderButton>
    <HeaderButton @click="() => window.maximize()">
      <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
        <rect
          x="6"
          y="6"
          width="12"
          height="12"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    </HeaderButton>
    <HeaderButton @click="() => window.close()">
      <XMarkIcon />
    </HeaderButton>
  </div>
</template>

<script setup lang="ts">
import {
  MinusIcon,
  XMarkIcon,
} from "@heroicons/vue/16/solid";
import { getCurrentWindow } from "@tauri-apps/api/window";

const window = getCurrentWindow();
</script>
