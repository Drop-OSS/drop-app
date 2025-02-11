<template>
  <div class="flex flex-row h-full">
    <div class="flex-none max-h-full overflow-y-auto w-64 bg-zinc-950 px-2 py-1">
      <ul class="flex flex-col gap-y-1">
        <NuxtLink v-for="(nav, navIdx) in navigation" :key="nav.route" :class="[
          'transition-all duration-200 rounded-lg flex items-center py-1.5 px-3',
          navIdx === currentNavigation
            ? 'bg-zinc-800 text-zinc-100'
            : nav.isInstalled.value
              ? 'text-zinc-300 hover:bg-zinc-800/90 hover:text-zinc-200'
              : 'text-zinc-500 hover:bg-zinc-800/70 hover:text-zinc-300',
        ]" :href="nav.route">
          <div class="flex items-center w-full gap-x-3">
            <img class="size-6 flex-none object-cover bg-zinc-900 rounded" :src="icons[nav.id]" alt="" />
            <p class="truncate text-sm font-display leading-6 flex-1">
              {{ nav.label }}
            </p>
          </div>
        </NuxtLink>
      </ul>
    </div>
    <div class="grow overflow-y-auto">
      <NuxtPage :libraryDownloadError="libraryDownloadError" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { GameStatusEnum, type Game, type GameStatus, type NavigationItem } from "~/types";

let libraryDownloadError = false;

const games: { [key: string]: { game: Game, status: Ref<GameStatus, GameStatus> } } = {};
const icons: { [key: string]: string } = {};

const rawGames: Ref<Game[], Game[]> = ref([])

async function calculateGames() {
  try {
    rawGames.value = await invoke("fetch_library");
    for (const game of rawGames.value) {
      if (games[game.id]) continue;
      games[game.id] = await useGame(game.id);
    }
    for (const game of rawGames.value) {
      if (icons[game.id]) continue;
      icons[game.id] = await useObject(game.mIconId);
    }
  }
  catch (e) {
    console.log(e)
    libraryDownloadError = true;
    return new Array();
  }
}

await calculateGames();

const navigation = computed(() => rawGames.value.map((game) => {
  const status = games[game.id].status;

  const isInstalled = computed(
    () =>
      status.value.type == GameStatusEnum.Installed ||
      status.value.type == GameStatusEnum.SetupRequired
  );

  const item = {
    label: game.mName,
    route: `/library/${game.id}`,
    prefix: `/library/${game.id}`,
    isInstalled,
    id: game.id,
  };
  return item;
})
);
const { currentNavigation, recalculateNavigation } = useCurrentNavigationIndex(navigation.value);


listen("update_library", async (event) => {
  console.log("Updating library");
  await calculateGames()
  recalculateNavigation();
})

</script>
