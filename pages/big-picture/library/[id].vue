<template>
  <div class="p-8">
    <!-- Back Button -->
    <div class="mb-8">
      <button
        @click="() => router.push('/big-picture/library')"
        class="flex items-center space-x-3 text-zinc-400 hover:text-zinc-200 transition-colors duration-200 text-lg"
      >
        <ArrowLeftIcon class="h-6 w-6" />
        <span>Back to Library</span>
      </button>
    </div>

    <!-- Loading State -->
    <div v-if="!game" class="flex items-center justify-center h-64">
      <div class="text-center">
        <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto mb-4"></div>
        <p class="text-zinc-400 text-lg">Loading game...</p>
      </div>
    </div>

    <!-- Game Content -->
    <div v-else class="space-y-8">
      <!-- Game Header -->
      <div class="flex items-start space-x-8">
        <!-- Game Cover -->
        <div class="flex-shrink-0">
          <img
            :src="gameCover"
            :alt="game.mName"
            class="w-64 h-48 rounded-xl object-cover shadow-lg"
          />
        </div>
        
        <!-- Game Info -->
        <div class="flex-1">
          <h1 class="text-5xl font-bold text-zinc-100 mb-4">
            {{ game.mName }}
          </h1>
          <p class="text-xl text-zinc-400 mb-6 leading-relaxed">
            {{ game.mShortDescription }}
          </p>
          
          <!-- Status Badge -->
          <div class="mb-8">
            <div
              :class="[
                'inline-flex px-6 py-3 rounded-full text-lg font-semibold',
                statusClasses[gameStatus.type]
              ]"
            >
              {{ statusLabels[gameStatus.type] }}
            </div>
          </div>
          
          <!-- Action Button -->
          <div>
            <button
              :class="[
                'px-12 py-6 rounded-xl font-semibold text-xl transition-all duration-200 transform hover:scale-105 focus:outline-none focus:ring-4 focus:ring-blue-600 focus:ring-offset-2 focus:ring-offset-zinc-950',
                actionButtonClasses[gameStatus.type]
              ]"
              @click="handleAction"
              :disabled="isActionLoading"
            >
              <div class="flex items-center justify-center space-x-4">
                <div v-if="isActionLoading" class="animate-spin rounded-full h-6 w-6 border-b-2 border-white"></div>
                <component v-else :is="actionIcons[gameStatus.type]" class="h-8 w-8" />
                <span>{{ actionLabels[gameStatus.type] }}</span>
              </div>
            </button>
          </div>
        </div>
      </div>

      <!-- Game Description -->
      <div v-if="game.mDescription" class="bg-zinc-800/50 rounded-xl p-8 backdrop-blur-sm">
        <h2 class="text-3xl font-semibold text-zinc-100 mb-6">About This Game</h2>
        <div class="prose prose-invert prose-blue max-w-none">
          <p class="text-zinc-300 leading-relaxed text-lg">
            {{ game.mDescription }}
          </p>
        </div>
      </div>

      <!-- Additional Actions -->
      <div class="grid grid-cols-2 gap-6">
        <!-- Store Button -->
        <button
          @click="openStore"
          class="p-6 bg-zinc-800 hover:bg-zinc-700 rounded-xl transition-all duration-200 transform hover:scale-105 focus:outline-none focus:ring-4 focus:ring-blue-600 focus:ring-offset-2 focus:ring-offset-zinc-950"
        >
          <div class="flex items-center justify-center space-x-3">
            <BuildingStorefrontIcon class="h-8 w-8 text-zinc-400" />
            <span class="text-xl font-semibold text-zinc-100">View in Store</span>
          </div>
        </button>

        <!-- Queue Button -->
        <button
          @click="router.push('/big-picture/queue')"
          class="p-6 bg-zinc-800 hover:bg-zinc-700 rounded-xl transition-all duration-200 transform hover:scale-105 focus:outline-none focus:ring-4 focus:ring-blue-600 focus:ring-offset-2 focus:ring-offset-zinc-950"
        >
          <div class="flex items-center justify-center space-x-3">
            <QueueListIcon class="h-8 w-8 text-zinc-400" />
            <span class="text-xl font-semibold text-zinc-100">View Downloads</span>
          </div>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ArrowLeftIcon } from "@heroicons/vue/24/outline";
import {
  PlayIcon,
  ArrowDownTrayIcon,
  QueueListIcon,
  StopIcon,
  WrenchIcon,
  BuildingStorefrontIcon,
} from "@heroicons/vue/20/solid";
import { GameStatusEnum, type GameStatus, type Game } from "~/types";
import { invoke } from "@tauri-apps/api/core";
import { useGame } from "~/composables/game";

definePageMeta({
  layout: "big-picture"
});

const route = useRoute();
const router = useRouter();
const gameId = route.params.id as string;

// Game data
const game = ref<Game | null>(null);
const gameStatus = ref<GameStatus>({ type: GameStatusEnum.Remote });
const gameCover = ref<string>("");
const isActionLoading = ref(false);

// Load game data
try {
  const { game: rawGame, status } = await useGame(gameId);
  game.value = rawGame;
  gameStatus.value = status.value;
  gameCover.value = await useObject(rawGame.mCoverObjectId);
} catch (error) {
  console.error("Failed to load game data:", error);
}

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
  [GameStatusEnum.Remote]: "Install Game",
  [GameStatusEnum.Queued]: "View Queue",
  [GameStatusEnum.Downloading]: "View Progress",
  [GameStatusEnum.Installed]: "Play Game",
  [GameStatusEnum.Running]: "Stop Game",
  [GameStatusEnum.SetupRequired]: "Setup Game",
  [GameStatusEnum.Updating]: "View Progress",
  [GameStatusEnum.Uninstalling]: "Uninstalling...",
  [GameStatusEnum.PartiallyInstalled]: "Resume Download"
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
  if (!game.value) return;
  
  isActionLoading.value = true;
  
  try {
    switch (gameStatus.value.type) {
      case GameStatusEnum.Remote:
        // Navigate to queue for installation
        await router.push("/big-picture/queue");
        break;
      case GameStatusEnum.Queued:
      case GameStatusEnum.Downloading:
      case GameStatusEnum.Updating:
        // Navigate to queue
        await router.push("/big-picture/queue");
        break;
      case GameStatusEnum.Installed:
        // Launch game
        await invoke("launch_game", { id: gameId });
        break;
      case GameStatusEnum.Running:
        // Stop game
        await invoke("kill_game", { gameId });
        break;
      case GameStatusEnum.SetupRequired:
        // Launch game for setup
        await invoke("launch_game", { id: gameId });
        break;
      case GameStatusEnum.PartiallyInstalled:
        // Resume download
        await invoke("resume_download", { gameId });
        break;
    }
  } catch (error) {
    console.error("Action failed:", error);
  } finally {
    isActionLoading.value = false;
  }
};

const openStore = () => {
  if (!game.value) return;
  
  // Open store URL in default browser
  invoke("open_url", { 
    url: `https://store.drop.com/game/${game.value.id}` 
  }).catch(console.error);
};
</script> 