<template>
  <NuxtLink
    :to="`/big-picture/library/${props.game.id}`"
    class="group relative bg-zinc-800 rounded-xl overflow-hidden cursor-pointer transition-all duration-200 transform hover:scale-105 focus:outline-none focus:ring-2 focus:ring-blue-600 focus:ring-offset-2 focus:ring-offset-zinc-900 block"
  >
    <!-- Game Cover Image -->
    <div class="aspect-[4/3] relative overflow-hidden">
      <img
        :src="coverUrl"
        :alt="game.mName"
        class="w-full h-full object-cover transition-transform duration-300 group-hover:scale-110"
      />
      
      <!-- Overlay -->
      <div class="absolute inset-0 bg-gradient-to-t from-black/60 via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-200" />
      
      <!-- Status Badge -->
      <div class="absolute top-3 right-3">
        <div
          :class="[
            'px-2 py-1 rounded-full text-xs font-semibold',
            statusClasses[status.type]
          ]"
        >
          {{ statusLabels[status.type] }}
        </div>
      </div>
    </div>
    
    <!-- Game Info -->
    <div class="p-4">
      <h3 class="text-lg font-semibold text-zinc-100 truncate mb-1">
        {{ game.mName }}
      </h3>
      <p class="text-sm text-zinc-400 line-clamp-2">
        {{ game.mShortDescription }}
      </p>
      
      <!-- Action Button -->
      <div class="mt-4">
        <button
          :class="[
            'w-full py-2 px-4 rounded-lg font-semibold transition-all duration-200 transform hover:scale-105 focus:outline-none focus:ring-2 focus:ring-blue-600',
            actionButtonClasses[status.type]
          ]"
          @click.stop="handleAction"
        >
          <div class="flex items-center justify-center space-x-2">
            <component :is="actionIcons[status.type]" class="h-4 w-4" />
            <span>{{ actionLabels[status.type] }}</span>
          </div>
        </button>
      </div>
    </div>
  </NuxtLink>
</template>

<script setup lang="ts">
import {
  PlayIcon,
  ArrowDownTrayIcon,
  QueueListIcon,
  StopIcon,
  WrenchIcon,
} from "@heroicons/vue/20/solid";
import { GameStatusEnum, type GameStatus, type Game } from "~/types";
import { invoke } from "@tauri-apps/api/core";

const props = defineProps<{
  game: Game;
  status?: GameStatus;
}>();

// Use provided status or default to Remote
const status = computed(() => props.status || { type: GameStatusEnum.Remote });

// Get cover image URL
const coverUrl = await useObject(props.game.mCoverObjectId);

// Status styling
const statusClasses = {
  [GameStatusEnum.Remote]: "bg-zinc-600 text-zinc-200",
  [GameStatusEnum.Queued]: "bg-blue-600 text-white",
  [GameStatusEnum.Downloading]: "bg-blue-600 text-white",
  [GameStatusEnum.Installed]: "bg-green-600 text-white",
  [GameStatusEnum.Running]: "bg-green-600 text-white",
  [GameStatusEnum.SetupRequired]: "bg-yellow-600 text-white",
  [GameStatusEnum.Updating]: "bg-blue-600 text-white",
  [GameStatusEnum.Uninstalling]: "bg-red-600 text-white",
  [GameStatusEnum.PartiallyInstalled]: "bg-gray-600 text-white"
};

const statusLabels = {
  [GameStatusEnum.Remote]: "Not Installed",
  [GameStatusEnum.Queued]: "Queued",
  [GameStatusEnum.Downloading]: "Downloading",
  [GameStatusEnum.Installed]: "Installed",
  [GameStatusEnum.Running]: "Running",
  [GameStatusEnum.SetupRequired]: "Setup Required",
  [GameStatusEnum.Updating]: "Updating",
  [GameStatusEnum.Uninstalling]: "Uninstalling",
  [GameStatusEnum.PartiallyInstalled]: "Partially Installed"
};

// Action button styling
const actionButtonClasses = {
  [GameStatusEnum.Remote]: "bg-blue-600 hover:bg-blue-500 text-white",
  [GameStatusEnum.Queued]: "bg-zinc-600 hover:bg-zinc-500 text-white",
  [GameStatusEnum.Downloading]: "bg-zinc-600 hover:bg-zinc-500 text-white",
  [GameStatusEnum.Installed]: "bg-green-600 hover:bg-green-500 text-white",
  [GameStatusEnum.Running]: "bg-red-600 hover:bg-red-500 text-white",
  [GameStatusEnum.SetupRequired]: "bg-yellow-600 hover:bg-yellow-500 text-white",
  [GameStatusEnum.Updating]: "bg-zinc-600 hover:bg-zinc-500 text-white",
  [GameStatusEnum.Uninstalling]: "bg-zinc-600 hover:bg-zinc-500 text-white",
  [GameStatusEnum.PartiallyInstalled]: "bg-blue-600 hover:bg-blue-500 text-white"
};

const actionLabels = {
  [GameStatusEnum.Remote]: "Install",
  [GameStatusEnum.Queued]: "View Queue",
  [GameStatusEnum.Downloading]: "View Progress",
  [GameStatusEnum.Installed]: "Play",
  [GameStatusEnum.Running]: "Stop",
  [GameStatusEnum.SetupRequired]: "Setup",
  [GameStatusEnum.Updating]: "View Progress",
  [GameStatusEnum.Uninstalling]: "Uninstalling...",
  [GameStatusEnum.PartiallyInstalled]: "Resume"
};

const actionIcons = {
  [GameStatusEnum.Remote]: ArrowDownTrayIcon,
  [GameStatusEnum.Queued]: QueueListIcon,
  [GameStatusEnum.Downloading]: ArrowDownTrayIcon,
  [GameStatusEnum.Installed]: PlayIcon,
  [GameStatusEnum.Running]: StopIcon,
  [GameStatusEnum.SetupRequired]: WrenchIcon,
  [GameStatusEnum.Updating]: ArrowDownTrayIcon,
  [GameStatusEnum.Uninstalling]: ArrowDownTrayIcon,
  [GameStatusEnum.PartiallyInstalled]: ArrowDownTrayIcon
};

const handleAction = async () => {
  // Handle different actions based on status
  switch (status.value.type) {
    case GameStatusEnum.Remote:
      // Navigate to game detail page for installation
      break;
    case GameStatusEnum.Queued:
    case GameStatusEnum.Downloading:
    case GameStatusEnum.Updating:
      // Navigate to queue
      await navigateTo("/big-picture/queue");
      break;
    case GameStatusEnum.Installed:
      // Launch game
      try {
        await invoke("launch_game", { id: props.game.id });
      } catch (error) {
        console.error("Failed to launch game:", error);
      }
      break;
    case GameStatusEnum.Running:
      // Stop game
      try {
        await invoke("kill_game", { gameId: props.game.id });
      } catch (error) {
        console.error("Failed to stop game:", error);
      }
      break;
    case GameStatusEnum.SetupRequired:
      // Launch game for setup
      try {
        await invoke("launch_game", { id: props.game.id });
      } catch (error) {
        console.error("Failed to launch game:", error);
      }
      break;
    case GameStatusEnum.PartiallyInstalled:
      // Resume download
      try {
        await invoke("resume_download", { gameId: props.game.id });
      } catch (error) {
        console.error("Failed to resume download:", error);
      }
      break;
  }
};
</script> 