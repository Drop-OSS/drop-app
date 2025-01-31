<template>
  <div class="flex flex-row h-full">
    <div
      class="flex-none max-h-full overflow-y-auto w-64 bg-zinc-950 px-2 py-1"
    >
      <ul class="flex flex-col gap-y-1">
        <NuxtLink
          v-for="(nav, navIdx) in navigation"
          :key="nav.route"
          :class="[
            'transition-all duration-200 rounded-lg flex items-center py-1.5 px-3',
            navIdx === currentNavigationIndex
              ? 'bg-zinc-800 text-zinc-100'
              : nav.isInstalled.value
              ? 'text-zinc-300 hover:bg-zinc-800/90 hover:text-zinc-200'
              : 'text-zinc-500 hover:bg-zinc-800/70 hover:text-zinc-300',
          ]"
          :href="nav.route"
        >
          <div class="flex items-center w-full gap-x-3">
            <img
              class="size-6 flex-none object-cover bg-zinc-900 rounded"
              :src="icons[navIdx]"
              alt=""
            />
            <p class="truncate text-sm font-display leading-6 flex-1">
              {{ nav.label }}
            </p>
          </div>
        </NuxtLink>
      </ul>
    </div>
    <div class="grow overflow-y-auto">
      <NuxtPage :libraryDownloadError = "libraryDownloadError" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { GameStatusEnum, type Game, type NavigationItem } from "~/types";

let libraryDownloadError = false;

async function calculateGames(): Promise<Game[]> {
  try {
    return await invoke("fetch_library");
  }
  catch(e) {
    console.log(e)
    libraryDownloadError = true;
    return new Array();
  }
}

const rawGames: Array<Game> = await calculateGames();
const games = await Promise.all(rawGames.map((e) => useGame(e.id)));
const icons = await Promise.all(
  games.map(({ game, status }) => useObject(game.mIconId))
);

const navigation = games.map(({ game, status }) => {
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
  };
  return item;
});

const currentNavigationIndex = useCurrentNavigationIndex(navigation);
</script>
