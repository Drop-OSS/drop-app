<template>
  <div class="h-full w-full p-8">
    <div class="max-w-7xl mx-auto">
      <div class="mb-8">
        <input
          type="text"
          v-model="searchQuery"
          class="w-full rounded-lg border-0 bg-zinc-800/50 py-4 px-6 text-zinc-100 placeholder:text-zinc-500 focus:bg-zinc-800 focus:ring-2 focus:ring-inset focus:ring-blue-500 text-lg"
          placeholder="Search games..."
        />
      </div>

      <TransitionGroup
        name="grid"
        tag="div"
        class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6"
      >
        <NuxtLink
          v-for="game in filteredGames"
          :key="game.id"
          :to="`/library/${game.id}`"
          class="group relative aspect-[4/3] rounded-xl overflow-hidden bg-zinc-800/50 hover:bg-zinc-800 transition-all duration-300 hover:scale-105 hover:shadow-xl hover:shadow-zinc-950/50"
        >
          <img
            :src="icons[game.id]"
            :alt="game.mName"
            class="w-full h-full object-cover"
          />
          <div class="absolute inset-0 bg-gradient-to-t from-zinc-950 via-zinc-950/50 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-300">
            <div class="absolute bottom-0 left-0 right-0 p-6">
              <h3 class="text-xl font-display font-semibold text-zinc-100 mb-2">
                {{ game.mName }}
              </h3>
              <p
                class="text-sm font-medium"
                :class="[gameStatusTextStyle[games[game.id].status.value.type]]"
              >
                {{ gameStatusText[games[game.id].status.value.type] }}
              </p>
            </div>
          </div>
        </NuxtLink>
      </TransitionGroup>
    </div>
  </div>
</template>

<script setup lang="ts">
import { GameStatusEnum, type Game, type GameStatus } from "~/types";
import { invoke } from "@tauri-apps/api/core";

definePageMeta({
  layout: 'big-picture'
});

const searchQuery = ref("");

const games: {
  [key: string]: { game: Game; status: Ref<GameStatus, GameStatus> };
} = {};
const icons: { [key: string]: string } = {};

const rawGames: Ref<Game[], Game[]> = ref([]);

// Style information
const gameStatusTextStyle: { [key in GameStatusEnum]: string } = {
  [GameStatusEnum.Installed]: "text-green-500",
  [GameStatusEnum.Downloading]: "text-blue-500",
  [GameStatusEnum.Running]: "text-green-500",
  [GameStatusEnum.Remote]: "text-zinc-500",
  [GameStatusEnum.Queued]: "text-blue-500",
  [GameStatusEnum.Updating]: "text-blue-500",
  [GameStatusEnum.Uninstalling]: "text-zinc-100",
  [GameStatusEnum.SetupRequired]: "text-yellow-500",
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
}

await calculateGames();

const filteredGames = computed(() => {
  if (!searchQuery.value) return rawGames.value;
  const query = searchQuery.value.toLowerCase();
  return rawGames.value.filter((game) =>
    game.mName.toLowerCase().includes(query)
  );
});
</script>

<style scoped>
.grid-move,
.grid-enter-active,
.grid-leave-active {
  transition: all 0.5s ease;
}

.grid-enter-from,
.grid-leave-to {
  opacity: 0;
  transform: scale(0.8);
}

.grid-leave-active {
  position: absolute;
}
</style> 