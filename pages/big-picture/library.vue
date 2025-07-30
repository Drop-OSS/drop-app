<template>
  <div class="p-8">
    <!-- Game Detail View -->
    <div v-if="selectedGame" class="space-y-8">
      <!-- Back Button -->
      <div class="mb-8">
        <button
          @click="() => selectedGame = null"
          class="flex items-center space-x-3 text-zinc-400 hover:text-zinc-200 transition-colors duration-200 text-lg"
        >
          <ArrowLeftIcon class="h-6 w-6" />
          <span>Back to Library</span>
        </button>
      </div>

      <!-- Game Header -->
      <div class="flex items-start space-x-8">
        <!-- Game Cover -->
        <div class="flex-shrink-0">
          <img
            :src="selectedGameCover"
            :alt="selectedGame.mName"
            class="w-64 h-48 rounded-xl object-cover shadow-lg"
          />
        </div>
        
        <!-- Game Info -->
        <div class="flex-1">
          <h1 class="text-5xl font-bold text-zinc-100 mb-4">
            {{ selectedGame.mName }}
          </h1>
          <p class="text-xl text-zinc-400 mb-6 leading-relaxed">
            {{ selectedGame.mShortDescription }}
          </p>
          
          <!-- Status Badge -->
          <div class="mb-8">
            <div
              :class="[
                'inline-flex px-6 py-3 rounded-full text-lg font-semibold',
                getStatusClasses(selectedGameStatus.type)
              ]"
            >
              {{ getStatusLabels(selectedGameStatus.type) }}
            </div>
          </div>
          
          <!-- Action Button -->
          <div>
            <button
              :class="[
                'px-12 py-6 rounded-xl font-semibold text-xl transition-all duration-200 transform hover:scale-105 focus:outline-none focus:ring-4 focus:ring-blue-600 focus:ring-offset-2 focus:ring-offset-zinc-950',
                getActionButtonClasses(selectedGameStatus.type)
              ]"
              @click="handleSelectedGameAction"
              :disabled="isActionLoading"
            >
              <div class="flex items-center justify-center space-x-4">
                <div v-if="isActionLoading" class="animate-spin rounded-full h-6 w-6 border-b-2 border-white"></div>
                <component v-else :is="getActionIcons(selectedGameStatus.type)" class="h-8 w-8" />
                <span>{{ getActionLabels(selectedGameStatus.type) }}</span>
              </div>
            </button>
          </div>
        </div>
      </div>

             <!-- Game Description -->
       <div v-if="selectedGame.mDescription" class="bg-zinc-800/50 rounded-xl p-8 backdrop-blur-sm">
         <h2 class="text-3xl font-semibold text-zinc-100 mb-6">About This Game</h2>
         <div class="prose prose-invert prose-blue max-w-none">
           <p class="text-zinc-300 leading-relaxed text-lg break-words overflow-hidden">
             {{ selectedGame.mDescription }}
           </p>
         </div>
       </div>

      <!-- Additional Actions -->
      <div class="flex justify-center">
        <!-- Store Button -->
        <a
          :href="selectedGameRemoteUrl"
          target="_blank"
          class="p-6 bg-zinc-800 hover:bg-zinc-700 rounded-xl transition-all duration-200 transform hover:scale-105 focus:outline-none focus:ring-4 focus:ring-blue-600 focus:ring-offset-2 focus:ring-offset-zinc-950"
        >
          <div class="flex items-center justify-center space-x-3">
            <BuildingStorefrontIcon class="h-8 w-8 text-zinc-400" />
            <span class="text-xl font-semibold text-zinc-100">View in Store</span>
          </div>
        </a>
      </div>
    </div>

    <!-- Library Grid View -->
    <div v-else>
      <!-- Page Header -->
      <div class="mb-8">
        <h2 class="text-3xl font-bold font-display text-zinc-100 mb-2">
          Your Library
        </h2>
        <p class="text-lg text-zinc-400">
          {{ games.length }} games in your collection
        </p>
      </div>

      <!-- Games Grid -->
      <div v-if="games.length > 0" class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
        <div
          v-for="gameData in games"
          :key="gameData.game.id"
          @click="selectGame(gameData.game)"
          class="group relative bg-zinc-800 rounded-xl overflow-hidden cursor-pointer transition-all duration-200 transform hover:scale-105 focus:outline-none focus:ring-2 focus:ring-blue-600 focus:ring-offset-2 focus:ring-offset-zinc-900 block"
        >
          <!-- Game Cover Image -->
          <div class="aspect-[4/3] relative overflow-hidden">
            <img
              :src="gameData.cover"
              :alt="gameData.game.mName"
              class="w-full h-full object-cover transition-transform duration-300 group-hover:scale-110"
            />
            
            <!-- Overlay -->
            <div class="absolute inset-0 bg-gradient-to-t from-black/60 via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-200" />
            
            <!-- Status Badge -->
            <div class="absolute top-3 right-3">
              <div
                :class="[
                  'px-2 py-1 rounded-full text-xs font-semibold',
                  getStatusClasses(gameData.status.type)
                ]"
              >
                {{ getStatusLabels(gameData.status.type) }}
              </div>
            </div>
          </div>
          
          <!-- Game Info -->
          <div class="p-4">
            <h3 class="text-lg font-semibold text-zinc-100 truncate mb-1">
              {{ gameData.game.mName }}
            </h3>
            <p class="text-sm text-zinc-400 line-clamp-2">
              {{ gameData.game.mShortDescription }}
            </p>
            
            <!-- Action Button -->
            <div class="mt-4">
              <button
                :class="[
                  'w-full py-2 px-4 rounded-lg font-semibold transition-all duration-200 transform hover:scale-105 focus:outline-none focus:ring-2 focus:ring-blue-600',
                  getActionButtonClasses(gameData.status.type)
                ]"
                @click.stop="handleGameAction(gameData.game, gameData.status)"
              >
                <div class="flex items-center justify-center space-x-2">
                  <component :is="getActionIcons(gameData.status.type)" class="h-4 w-4" />
                  <span>{{ getActionLabels(gameData.status.type) }}</span>
                </div>
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Empty State -->
      <div v-else class="flex flex-col items-center justify-center py-16">
        <div class="text-center">
          <BookOpenIcon class="h-24 w-24 text-zinc-600 mx-auto mb-6" />
          <h3 class="text-2xl font-semibold text-zinc-300 mb-2">
            No games in your library
          </h3>
          <p class="text-zinc-500 mb-6">
            Visit the store to discover and install games
          </p>
          <NuxtLink
            to="/big-picture/store"
            class="inline-flex items-center px-6 py-3 bg-blue-600 hover:bg-blue-500 text-white font-semibold rounded-lg transition-all duration-200 transform hover:scale-105"
          >
            <BuildingStorefrontIcon class="h-5 w-5 mr-2" />
            Visit Store
          </NuxtLink>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { BookOpenIcon, BuildingStorefrontIcon, ArrowLeftIcon } from "@heroicons/vue/24/outline";
import {
  PlayIcon,
  ArrowDownTrayIcon,
  QueueListIcon,
  StopIcon,
  WrenchIcon,
} from "@heroicons/vue/20/solid";
import { invoke } from "@tauri-apps/api/core";
import { GameStatusEnum, type Game, type GameStatus } from "~/types";

definePageMeta({
  layout: "big-picture"
});

const router = useRouter();

// Fetch games with status from backend
const games = ref<{ game: Game; status: GameStatus; cover: string }[]>([]);

// Selected game state
const selectedGame = ref<Game | null>(null);
const selectedGameStatus = ref<GameStatus>({ type: GameStatusEnum.Remote });
const selectedGameCover = ref<string>("");
const selectedGameRemoteUrl = ref<string>("");
const isActionLoading = ref(false);

onMounted(async () => {
  try {
    const libraryData = await invoke("fetch_library");
    const gameIds = libraryData as Game[];
    
    // Load each game with its status and cover
    const gamesWithStatus = await Promise.all(
      gameIds.map(async (game) => {
        const gameData = await useGame(game.id);
        const cover = await useObject(gameData.game.mCoverObjectId);
        return {
          game: gameData.game,
          status: gameData.status.value,
          cover
        };
      })
    );
    
    games.value = gamesWithStatus;
  } catch (error) {
    console.error("Failed to fetch library:", error);
  }
});

// Navigation functions
const navigateToQueue = () => {
  router.push('/big-picture/queue');
};

const selectGame = async (game: Game) => {
  selectedGame.value = game;
  
  // Load game status, cover, and remote URL
  try {
    const gameData = await useGame(game.id);
    selectedGameStatus.value = gameData.status.value;
    selectedGameCover.value = await useObject(game.mCoverObjectId);
    selectedGameRemoteUrl.value = await invoke("gen_drop_url", {
      path: `/store/${game.id}`,
    });
  } catch (error) {
    console.error("Failed to load selected game data:", error);
  }
};

// Status styling functions
const getStatusClasses = (statusType: GameStatusEnum) => {
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
  return statusClasses[statusType];
};

const getStatusLabels = (statusType: GameStatusEnum) => {
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
  return statusLabels[statusType];
};

// Action button styling functions
const getActionButtonClasses = (statusType: GameStatusEnum) => {
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
  return actionButtonClasses[statusType];
};

const getActionLabels = (statusType: GameStatusEnum) => {
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
  return actionLabels[statusType];
};

const getActionIcons = (statusType: GameStatusEnum) => {
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
  return actionIcons[statusType];
};

// Game action handlers
const handleSelectedGameAction = async () => {
  if (!selectedGame.value) return;
  
  isActionLoading.value = true;
  
  try {
    switch (selectedGameStatus.value.type) {
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
        await invoke("launch_game", { id: selectedGame.value.id });
        break;
      case GameStatusEnum.Running:
        // Stop game
        await invoke("kill_game", { gameId: selectedGame.value.id });
        break;
      case GameStatusEnum.SetupRequired:
        // Launch game for setup
        await invoke("launch_game", { id: selectedGame.value.id });
        break;
      case GameStatusEnum.PartiallyInstalled:
        // Resume download
        await invoke("resume_download", { gameId: selectedGame.value.id });
        break;
    }
  } catch (error) {
    console.error("Action failed:", error);
  } finally {
    isActionLoading.value = false;
  }
};

const handleGameAction = async (game: Game, status: GameStatus) => {
  switch (status.type) {
    case GameStatusEnum.Remote:
      // Select the game to show detail view
      await selectGame(game);
      break;
    case GameStatusEnum.Queued:
    case GameStatusEnum.Downloading:
    case GameStatusEnum.Updating:
      // Navigate to queue
      router.push("/big-picture/queue");
      break;
    case GameStatusEnum.Installed:
      // Launch game
      try {
        await invoke("launch_game", { id: game.id });
      } catch (error) {
        console.error("Failed to launch game:", error);
      }
      break;
    case GameStatusEnum.Running:
      // Stop game
      try {
        await invoke("kill_game", { gameId: game.id });
      } catch (error) {
        console.error("Failed to stop game:", error);
      }
      break;
    case GameStatusEnum.SetupRequired:
      // Launch game for setup
      try {
        await invoke("launch_game", { id: game.id });
      } catch (error) {
        console.error("Failed to launch game:", error);
      }
      break;
    case GameStatusEnum.PartiallyInstalled:
      // Resume download
      try {
        await invoke("resume_download", { gameId: game.id });
      } catch (error) {
        console.error("Failed to resume download:", error);
      }
      break;
  }
};



</script> 