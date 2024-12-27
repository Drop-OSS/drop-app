<template>
  <div class="flex flex-row h-full">
    <div class="flex-none max-h-full overflow-y-auto w-64 bg-zinc-950 px-2 py-1">
      <ul class="flex flex-col gap-y-1">
        <NuxtLink
          v-for="(nav, navIdx) in navigation"
          :key="nav.route"
          :class="[
            'transition-all duration-200 rounded-lg flex items-center py-1.5 px-3',
            navIdx === currentNavigationIndex 
              ? 'bg-zinc-800 text-zinc-100' 
              : 'bg-zinc-900/50 text-zinc-400 hover:bg-zinc-800/70 hover:text-zinc-300',
          ]"
          :href="nav.route"
        >
          <div class="flex items-center w-full gap-x-3">
            <img
              class="h-8 w-10 flex-none object-cover bg-zinc-900"
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
      <NuxtPage />
    </div>
  </div>
</template>

<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import type { Game, NavigationItem } from "~/types";

const games: Array<Game> = await invoke("fetch_library");
const icons = await Promise.all(games.map((e) => useObject(e.mIconId)));

const navigation = games.map((e) => {
  const item: NavigationItem = {
    label: e.mName,
    route: `/library/${e.id}`,
    prefix: `/library/${e.id}`,
  };
  return item;
});

const currentNavigationIndex = useCurrentNavigationIndex(navigation);
</script>
