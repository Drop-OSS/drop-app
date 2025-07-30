<template>
  <div class="h-screen w-screen">
         <!-- Game Detail View -->
     <div v-if="selectedGame" class="relative flex flex-col justify-center pt-72 overflow-hidden min-h-screen">
       <!-- Background Banner -->
       <div class="absolute inset-0 z-0">
         <img
           :src="selectedGameCover"
           class="w-full h-[24rem] object-cover blur-sm scale-105"
         />
         <div class="absolute inset-0 bg-gradient-to-t from-zinc-900 via-zinc-900/80 to-transparent opacity-90" />
         <div class="absolute inset-0 bg-gradient-to-r from-zinc-900/95 via-zinc-900/80 to-transparent opacity-90" />
       </div>

       <!-- Back Button -->
       <div class="relative z-10 px-8 mb-4">
         <button
           @click="() => selectedGame = null"
           class="flex items-center space-x-3 text-zinc-400 hover:text-zinc-200 transition-all duration-300 text-lg hover:scale-105"
         >
           <ArrowLeftIcon class="h-6 w-6" />
           <span>Back to Library</span>
         </button>
       </div>

       <!-- Game Header -->
       <div class="relative z-10 px-8 pb-4">
         <h1 class="text-6xl text-zinc-100 font-bold font-display drop-shadow-lg mb-8 leading-tight">
           {{ selectedGame.mName }}
         </h1>

         <div class="flex flex-row gap-x-6 items-stretch mb-8">
           <!-- Primary Action Button with Dropdown -->
           <div class="inline-flex divide-x divide-zinc-900">
             <button
               :class="[
                 'transition-all duration-300 hover:scale-105 active:scale-95 inline-flex items-center rounded-l-xl px-8 py-4 font-semibold text-xl shadow-xl backdrop-blur-sm uppercase font-display',
                 getActionButtonClasses(selectedGameStatus.type),
                 showSelectedGameDropdown ? 'rounded-l-xl' : 'rounded-xl'
               ]"
               @click="handleSelectedGameAction"
               :disabled="isActionLoading"
             >
               <div class="flex items-center justify-center space-x-3">
                 <div v-if="isActionLoading" class="animate-spin rounded-full h-6 w-6 border-b-2 border-white"></div>
                 <component v-else :is="getActionIcons(selectedGameStatus.type)" class="h-6 w-6" />
                 <span>{{ getActionLabels(selectedGameStatus.type) }}</span>
               </div>
             </button>
             
             <Menu v-if="showSelectedGameDropdown" as="div" class="relative inline-block text-left">
               <MenuButton :class="[
                 getActionButtonClasses(selectedGameStatus.type),
                 'transition-all duration-300 hover:scale-105 active:scale-95 inline-flex h-full justify-center items-center rounded-r-xl px-4 py-4 font-semibold text-xl shadow-xl backdrop-blur-sm uppercase font-display'
               ]">
                 <ChevronDownIcon class="h-6 w-6" aria-hidden="true" />
               </MenuButton>

               <transition enter-active-class="transition ease-out duration-100" enter-from-class="transform opacity-0 scale-95"
                 enter-to-class="transform opacity-100 scale-100" leave-active-class="transition ease-in duration-75"
                 leave-from-class="transform opacity-100 scale-100" leave-to-class="transform opacity-0 scale-95">
                 <MenuItems class="absolute right-0 z-[999] mt-2 w-40 origin-top-right rounded-xl bg-zinc-900/95 backdrop-blur-sm shadow-2xl ring-1 ring-zinc-100/5 focus:outline-none">
                   <div class="py-2">
                     <MenuItem v-slot="{ active }">
                       <button @click="handleSelectedGameOptions" :class="[
                         active ? 'bg-zinc-800 text-zinc-100' : 'text-zinc-400',
                         'w-full block px-4 py-3 text-lg font-semibold inline-flex justify-between items-center'
                       ]">
                         Options
                         <Cog6ToothIcon class="h-5 w-5" />
                       </button>
                     </MenuItem>
                     <MenuItem v-slot="{ active }">
                       <button @click="handleSelectedGameUninstall" :class="[
                         active ? 'bg-zinc-800 text-zinc-100' : 'text-zinc-400',
                         'w-full block px-4 py-3 text-lg font-semibold inline-flex justify-between items-center'
                       ]">
                         Uninstall
                         <TrashIcon class="h-5 w-5" />
                       </button>
                     </MenuItem>
                   </div>
                 </MenuItems>
               </transition>
             </Menu>
           </div>

           <!-- Store Button -->
           <a
             :href="selectedGameRemoteUrl"
             target="_blank"
             class="transition-all duration-300 hover:scale-105 active:scale-95 inline-flex items-center rounded-xl bg-zinc-800/50 px-6 py-4 font-semibold text-white shadow-xl backdrop-blur-sm hover:bg-zinc-800/80 uppercase font-display"
           >
             <BuildingStorefrontIcon class="mr-3 h-6 w-6" aria-hidden="true" />
             Store
           </a>
         </div>
       </div>

       <!-- Main Content -->
       <div class="relative z-10 w-full bg-zinc-900 px-8 py-8">
         <div class="max-w-7xl mx-auto">
           <div class="grid grid-cols-[2fr,1fr] gap-8">
                                     <!-- Game Description -->
                        <div class="space-y-6 relative z-0">
                          <div class="bg-zinc-800/50 rounded-xl p-8 backdrop-blur-sm w-full max-w-full overflow-hidden">
                 <h2 class="text-2xl font-display font-semibold text-zinc-100 mb-6">
                   About This Game
                 </h2>
                 <div class="prose prose-invert prose-blue overflow-y-auto custom-scrollbar max-w-none w-full overflow-hidden">
                   <p class="text-zinc-300 leading-relaxed text-lg break-words overflow-hidden w-full">
                     {{ selectedGame.mDescription }}
                   </p>
                 </div>
               </div>
             </div>

             <!-- Game Cover and Status -->
             <div class="space-y-6">
               <div class="bg-zinc-800/50 rounded-xl p-6 backdrop-blur-sm">
                 <h2 class="text-xl font-display font-semibold text-zinc-100 mb-4">
                   Game Status
                 </h2>
                 <div class="space-y-4">
                   <!-- Status Badge -->
                   <div class="flex items-center justify-between">
                     <span class="text-zinc-400">Status</span>
                     <div
                       :class="[
                         'inline-flex px-4 py-2 rounded-full text-sm font-semibold',
                         getStatusClasses(selectedGameStatus.type)
                       ]"
                     >
                       {{ getStatusLabels(selectedGameStatus.type) }}
                     </div>
                   </div>

                   <!-- Game Cover -->
                   <div class="relative aspect-[4/3] rounded-lg overflow-hidden">
                     <img
                       :src="selectedGameCover"
                       :alt="selectedGame.mName"
                       class="w-full h-full object-cover transition-transform duration-300 hover:scale-110"
                     />
                     <div class="absolute inset-0 bg-gradient-to-t from-black/20 to-transparent opacity-0 hover:opacity-100 transition-opacity duration-300" />
                   </div>
                 </div>
               </div>
             </div>
           </div>
         </div>
       </div>
     </div>

         <!-- Library Grid View -->
     <div v-else class="h-full w-full p-8">
       <!-- Page Header -->
       <div class="mb-8">
         <h2 class="text-4xl font-bold font-display text-zinc-100 mb-2">
           Your Library
         </h2>
         <p class="text-xl text-zinc-400">
           {{ games.length }} games in your collection
         </p>
       </div>

             <!-- Games Grid -->
       <div v-if="games.length > 0" class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-5 gap-8">
                 <div
           v-for="gameData in games"
           :key="gameData.game.id"
           @click="selectGame(gameData.game)"
           class="group relative bg-zinc-800/50 backdrop-blur-sm rounded-xl overflow-hidden cursor-pointer transition-all duration-300 transform hover:scale-110 focus:outline-none focus:ring-4 focus:ring-blue-600 focus:ring-offset-2 focus:ring-offset-zinc-900 block shadow-xl hover:shadow-2xl"
         >
                     <!-- Game Cover Image -->
           <div class="aspect-[4/3] relative overflow-hidden">
             <img
               :src="gameData.cover"
               :alt="gameData.game.mName"
               class="w-full h-full object-cover transition-transform duration-500 group-hover:scale-125"
             />
            
                         <!-- Overlay -->
             <div class="absolute inset-0 bg-gradient-to-t from-black/80 via-black/20 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-300" />
            
                         <!-- Status Badge -->
             <div class="absolute top-4 right-4">
               <div
                 :class="[
                   'px-3 py-2 rounded-full text-sm font-semibold shadow-lg backdrop-blur-sm',
                   getStatusClasses(gameData.status.type)
                 ]"
               >
                 {{ getStatusLabels(gameData.status.type) }}
               </div>
             </div>
          </div>
          
                     <!-- Game Info -->
           <div class="p-6">
             <h3 class="text-xl font-semibold text-zinc-100 truncate mb-2">
               {{ gameData.game.mName }}
             </h3>
             <p class="text-base text-zinc-400 line-clamp-2 leading-relaxed">
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
import { BookOpenIcon, BuildingStorefrontIcon, ArrowLeftIcon, Cog6ToothIcon, TrashIcon } from "@heroicons/vue/24/outline";
import {
  PlayIcon,
  ArrowDownTrayIcon,
  QueueListIcon,
  StopIcon,
  WrenchIcon,
  ChevronDownIcon,
} from "@heroicons/vue/20/solid";
import { Menu, MenuButton, MenuItem, MenuItems } from "@headlessui/vue";
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

// Computed properties
const showSelectedGameDropdown = computed(() => {
  if (!selectedGame.value) return false;
  return selectedGameStatus.value.type === GameStatusEnum.Installed ||
         selectedGameStatus.value.type === GameStatusEnum.SetupRequired ||
         selectedGameStatus.value.type === GameStatusEnum.PartiallyInstalled;
});

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

const handleSelectedGameOptions = async () => {
  if (!selectedGame.value) return;
  
  try {
    // For now, we'll just show a notification that options are not implemented in Big Picture mode
    console.log("Game options not implemented in Big Picture mode yet");
    // TODO: Implement game options modal for Big Picture mode
  } catch (error) {
    console.error("Options action failed:", error);
  }
};

const handleSelectedGameUninstall = async () => {
  if (!selectedGame.value) return;
  
  try {
    await invoke("uninstall_game", { gameId: selectedGame.value.id });
  } catch (error) {
    console.error("Uninstall failed:", error);
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

<style scoped>
.custom-scrollbar {
  scrollbar-width: thin;
  scrollbar-color: rgb(82 82 91) transparent;
}

.custom-scrollbar::-webkit-scrollbar {
  width: 6px;
}

.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background-color: rgb(82 82 91);
  border-radius: 3px;
}
</style> 