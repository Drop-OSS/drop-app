<template>
  <div class="bg-zinc-800 rounded-xl p-6">
    <div class="flex items-center space-x-6">
      <!-- Game Cover -->
      <div class="flex-shrink-0">
        <img
          :src="gameCover"
          :alt="gameName"
          class="w-24 h-24 rounded-lg object-cover"
        />
      </div>
      
      <!-- Game Info -->
      <div class="flex-1 min-w-0">
        <h3 class="text-xl font-semibold text-zinc-100 truncate mb-2">
          {{ gameName }}
        </h3>
        <p class="text-sm text-zinc-400 mb-4">
          {{ gameDescription }}
        </p>
        
        <!-- Progress Bar -->
        <div class="mb-4">
          <div class="flex justify-between text-sm text-zinc-400 mb-2">
            <span>{{ item.status }}</span>
            <span v-if="item.progress !== null">
              {{ Math.round(item.progress * 100) }}%
            </span>
          </div>
          <div class="w-full bg-zinc-700 rounded-full h-3 overflow-hidden">
            <div
              v-if="item.progress !== null"
              class="bg-blue-600 h-full rounded-full transition-all duration-300"
              :style="{ width: `${item.progress * 100}%` }"
            />
          </div>
        </div>
        
        <!-- Download Stats -->
        <div class="flex items-center justify-between text-sm text-zinc-400">
          <span>
            {{ formatBytes(item.current) }} / {{ formatBytes(item.max) }}
          </span>
          <span class="flex items-center">
            <ArrowDownTrayIcon class="h-4 w-4 mr-1" />
            Downloading
          </span>
        </div>
      </div>
      
      <!-- Actions -->
      <div class="flex-shrink-0 flex items-center space-x-3">
        <!-- Cancel Button -->
        <button
          @click="$emit('cancel', item.meta)"
          class="p-3 bg-red-600 hover:bg-red-500 text-white rounded-lg transition-all duration-200 transform hover:scale-105 focus:outline-none focus:ring-2 focus:ring-red-500 focus:ring-offset-2 focus:ring-offset-zinc-800"
        >
          <XMarkIcon class="h-5 w-5" />
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ArrowDownTrayIcon, XMarkIcon } from "@heroicons/vue/20/solid";
import { invoke } from "@tauri-apps/api/core";
import { type DownloadableMetadata, type Game } from "~/types";
import { useGame } from "~/composables/game";

const props = defineProps<{
  item: {
    meta: DownloadableMetadata;
    status: string;
    progress: number | null;
    current: number;
    max: number;
  };
  index: number;
}>();

defineEmits<{
  cancel: [meta: DownloadableMetadata];
  move: [oldIndex: number, newIndex: number];
}>();

// Get game data using the same pattern as the real queue page
const gameData = ref<{ game: Game; status: any; cover: string } | null>(null);

onMounted(async () => {
  try {
    const gameInfo = await useGame(props.item.meta.id);
    const cover = await useObject(gameInfo.game.mCoverObjectId);
    gameData.value = { ...gameInfo, cover };
  } catch (error) {
    console.error("Failed to fetch game data:", error);
  }
});

const gameName = computed(() => gameData.value?.game.mName || "Loading...");
const gameDescription = computed(() => gameData.value?.game.mShortDescription || "");
const gameCover = computed(() => gameData.value?.cover || "");

const formatBytes = (bytes: number): string => {
  const units = ["B", "KB", "MB", "GB", "TB"];
  let value = bytes;
  let unitIndex = 0;
  
  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024;
    unitIndex++;
  }
  
  return `${value.toFixed(1)} ${units[unitIndex]}`;
};
</script> 