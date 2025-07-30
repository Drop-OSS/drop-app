<template>
  <HeaderWidget @click="toggleBigPictureMode">
    <ComputerDesktopIcon 
      :class="[
        'h-5 transition-all duration-200',
        isBigPictureMode 
          ? 'text-blue-400 hover:text-blue-300' 
          : 'text-zinc-600 hover:text-zinc-300'
      ]" 
    />
  </HeaderWidget>
</template>

<script setup lang="ts">
import { ComputerDesktopIcon } from "@heroicons/vue/20/solid";
import HeaderWidget from "./HeaderWidget.vue";
import { useBigPictureMode, toggleBigPictureMode } from "~/composables/big-picture";

const state = useBigPictureMode();
const isBigPictureMode = computed(() => state.value.isActive);
</script> 