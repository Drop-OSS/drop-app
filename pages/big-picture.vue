<template>
  <div class="h-full w-full">
    <Transition name="fade" mode="out-in">
      <div v-if="!selectedGame" class="h-full w-full">
        <!-- Background gradient -->
        <div class="absolute inset-0">
          <div class="absolute inset-0 bg-gradient-to-b from-blue-950/40 via-blue-950/20 to-zinc-950"></div>
          <div class="absolute inset-0 bg-[radial-gradient(circle_at_center,_var(--tw-gradient-stops))] from-blue-500/10 via-transparent to-transparent"></div>
          <div class="absolute inset-0 bg-[radial-gradient(circle_at_50%_120%,_var(--tw-gradient-stops))] from-blue-600/20 via-transparent to-transparent"></div>
        </div>
        
        <div class="relative h-full flex flex-col">
          <!-- Header -->
          <div class="p-8 pb-4">
            <div class="flex items-center gap-4 mb-2">
              <Squares2X2Icon class="h-8 w-8 text-blue-400 drop-shadow" />
              <h1 class="text-4xl font-display font-bold text-zinc-100">Your Library</h1>
            </div>
            <p class="text-lg text-zinc-400">Browse and launch your games</p>
          </div>

          <!-- Search -->
          <div class="px-8 max-w-4xl w-full mx-auto mb-8">
            <div class="relative flex items-center">
              <span class="absolute inset-y-0 left-0 flex items-center pl-4 z-20">
                <span class="bg-zinc-800 rounded-full p-1"><MagnifyingGlassIcon class="h-5 w-5 text-zinc-400" /></span>
              </span>
              <input
                type="text"
                v-model="searchQuery"
                class="w-full rounded-xl border-0 bg-zinc-800/50 py-4 pl-12 pr-4 text-zinc-100 placeholder:text-zinc-500 focus:bg-zinc-800 focus:ring-2 focus:ring-inset focus:ring-blue-500 text-lg backdrop-blur-sm relative z-10"
                placeholder="Search your games..."
              />
            </div>
          </div>

          <!-- Game Grid -->
          <div class="flex-1 px-4 pb-8 overflow-y-auto">
            <TransitionGroup
              name="grid"
              tag="div"
              class="grid grid-cols-2 md:grid-cols-3 xl:grid-cols-4 gap-8"
            >
              <button
                v-for="game in filteredGames"
                :key="game.id"
                @click="selectGame(game)"
                class="group relative aspect-[16/9] w-full rounded-xl overflow-hidden bg-zinc-800/50 hover:bg-zinc-800 transition-all duration-300 hover:scale-105 hover:shadow-xl hover:shadow-zinc-950/50 focus:outline-none focus:ring-2 focus:ring-blue-500 backdrop-blur-sm"
              >
                <img
                  :src="banners[game.id]"
                  :alt="game.mName"
                  class="w-full h-full object-cover transition-transform duration-500 group-hover:scale-110"
                />
                <div class="absolute inset-0 bg-gradient-to-t from-zinc-950 via-zinc-950/50 to-transparent opacity-0 group-hover:opacity-100 transition-all duration-300">
                  <div class="absolute bottom-0 left-0 right-0 p-6">
                    <h3 class="text-xl font-display font-semibold text-zinc-100 mb-2 transform translate-y-2 group-hover:translate-y-0 transition-transform duration-300">
                      {{ game.mName }}
                    </h3>
                    <div class="flex items-center gap-2 transform translate-y-2 group-hover:translate-y-0 transition-transform duration-300 delay-75">
                      <span
                        class="px-2 py-1 rounded-full text-xs font-medium"
                        :class="[gameStatusTextStyle[games[game.id].status.value.type]]"
                      >
                        {{ gameStatusText[games[game.id].status.value.type] }}
                      </span>
                      <button
                        v-if="games[game.id].status.value.type === GameStatusEnum.Installed"
                        class="px-3 py-1 rounded-full text-xs font-medium bg-blue-600/20 text-blue-400 hover:bg-blue-600/30 transition-colors duration-300"
                        @click.stop="launchGame(game)"
                      >
                        Play
                      </button>
                    </div>
                  </div>
                </div>
              </button>
            </TransitionGroup>
          </div>
        </div>
      </div>

      <BigPictureGameDetails
        v-else
        :game="selectedGame"
        :games="games"
        :icons="icons"
        @back="selectedGame = null"
      />
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { MagnifyingGlassIcon, Squares2X2Icon } from '@heroicons/vue/24/outline';
import { GameStatusEnum, type Game, type GameStatus } from "~/types";
import { invoke } from "@tauri-apps/api/core";

definePageMeta({
  layout: 'big-picture'
});

const searchQuery = ref("");
const selectedGame = ref<Game | null>(null);

const games: {
  [key: string]: { game: Game; status: Ref<GameStatus, GameStatus> };
} = {};
const icons: { [key: string]: string } = {};
const banners: { [key: string]: string } = {};

const rawGames: Ref<Game[], Game[]> = ref([]);

// Style information
const gameStatusTextStyle: { [key in GameStatusEnum]: string } = {
  [GameStatusEnum.Installed]: "bg-green-500/20 text-green-400",
  [GameStatusEnum.Downloading]: "bg-blue-500/20 text-blue-400",
  [GameStatusEnum.Running]: "bg-green-500/20 text-green-400",
  [GameStatusEnum.Remote]: "bg-zinc-500/20 text-zinc-400",
  [GameStatusEnum.Queued]: "bg-blue-500/20 text-blue-400",
  [GameStatusEnum.Updating]: "bg-blue-500/20 text-blue-400",
  [GameStatusEnum.Uninstalling]: "bg-zinc-500/20 text-zinc-400",
  [GameStatusEnum.SetupRequired]: "bg-yellow-500/20 text-yellow-400",
};

const gameStatusText: { [key in GameStatusEnum]: string } = {
  [GameStatusEnum.Remote]: "Not installed",
  [GameStatusEnum.Queued]: "Queued",
  [GameStatusEnum.Downloading]: "Downloading...",
  [GameStatusEnum.Installed]: "Installed",
  [GameStatusEnum.Updating]: "Updating...",
  [GameStatusEnum.Uninstalling]: "Uninstalling...",
  [GameStatusEnum.SetupRequired]: "Setup required",
  [GameStatusEnum.Running]: "Running",
};

async function calculateGames() {
  rawGames.value = await invoke("fetch_library");
  for (const game of rawGames.value) {
    if (games[game.id]) continue;
    games[game.id] = await useGame(game.id);
  }
  for (const game of rawGames.value) {
    if (icons[game.id]) continue;
    icons[game.id] = await useObject(game.mIconObjectId);
  }
  for (const game of rawGames.value) {
    if (banners[game.id]) continue;
    banners[game.id] = await useObject(game.mBannerObjectId);
  }
}

await calculateGames();

const filteredGames = computed(() => {
  if (!searchQuery.value) return rawGames.value;
  const query = searchQuery.value.toLowerCase();
  return rawGames.value.filter((game) =>
    game.mName.toLowerCase().includes(query)
  );
});

const selectGame = (game: Game) => {
  selectedGame.value = game;
};

const launchGame = async (game: Game) => {
  await invoke("launch_game", { gameId: game.id });
};
</script>

<style scoped>
.grid-move,
.grid-enter-active,
.grid-leave-active {
  transition: all 0.5s cubic-bezier(0.4, 0, 0.2, 1);
}

.grid-enter-from,
.grid-leave-to {
  opacity: 0;
  transform: scale(0.8) translateY(20px);
}

.grid-leave-active {
  position: absolute;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style> 