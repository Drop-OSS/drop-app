<template>
  <div class="bg-zinc-900 border-b border-zinc-800 px-6 py-4 flex items-center justify-between transition-all duration-300">
    <!-- Left Section -->
    <div class="flex items-center space-x-6">
      <!-- Current Page Title -->
      <div class="transform transition-all duration-300">
        <h1 class="text-2xl font-bold font-display text-zinc-100">
          {{ currentPageTitle }}
        </h1>
        <p class="text-sm text-zinc-400 mt-1">
          {{ currentPageDescription }}
        </p>
      </div>
    </div>
    
    <!-- Right Section -->
    <div class="flex items-center space-x-4">
      <!-- User Info -->
      <div v-if="state.user" class="flex items-center space-x-3">
        <img 
          :src="profilePictureUrl" 
          class="w-10 h-10 rounded-lg border-2 border-zinc-700" 
          alt="Profile"
        />
        <div class="text-right">
          <p class="text-sm font-semibold text-zinc-100">
            {{ state.user.displayName }}
          </p>
          <p class="text-xs text-zinc-400">
            {{ state.user.username }}
          </p>
        </div>
      </div>
      
      <!-- Exit Button -->
      <button
        @click="exitBigPictureMode"
        class="px-6 py-3 bg-red-600 hover:bg-red-500 text-white font-semibold rounded-lg transition-all duration-200 transform hover:scale-105 focus:outline-none focus:ring-2 focus:ring-red-500 focus:ring-offset-2 focus:ring-offset-zinc-900"
      >
        <div class="flex items-center space-x-2">
          <XMarkIcon class="h-5 w-5" />
          <span>Exit</span>
        </div>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { XMarkIcon } from "@heroicons/vue/24/outline";
import { useBigPictureMode, exitBigPictureMode } from "~/composables/big-picture";
import { useAppState } from "~/composables/app-state";

const bigPictureState = useBigPictureMode();
const currentPage = computed(() => bigPictureState.value.currentPage);
const state = useAppState();

const profilePictureUrl: string = await useObject(
  state.value.user?.profilePictureObjectId ?? ""
);

const currentPageTitle = computed(() => {
  switch (currentPage.value) {
    case "/big-picture/library":
      return "Library";
    case "/big-picture/store":
      return "Store";
    case "/big-picture/queue":
      return "Downloads";
    case "/big-picture/settings":
      return "Settings";
    default:
      return "Drop";
  }
});

const currentPageDescription = computed(() => {
  switch (currentPage.value) {
    case "/big-picture/library":
      return "Your game collection";
    case "/big-picture/store":
      return "Discover new games";
    case "/big-picture/queue":
      return "Manage downloads";
    case "/big-picture/settings":
      return "App configuration";
    default:
      return "Game distribution platform";
  }
});
</script> 