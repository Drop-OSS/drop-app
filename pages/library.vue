<template>
  <div class="flex flex-row h-full">
    <div class="flex-none h-full w-64 bg-zinc-950 px-2 py-1">
      <ul class="flex flex-col gap-y-1">
        <NuxtLink
          v-for="(nav, navIdx) in navigation"
          :key="nav.route"
          :class="[
            'transition group rounded flex justify-between gap-x-6 py-2 px-3',
            navIdx === currentNavigationIndex ? 'bg-zinc-900' : '',
          ]"
          :href="nav.route"
        >
          <div class="flex items-center min-w-0 gap-x-2">
            <img
              class="h-5 w-auto flex-none object-cover rounded-sm bg-zinc-900"
              :src="icons[navIdx]"
              alt=""
            />
            <div class="min-w-0 flex-auto">
              <p
                :class="[
                  navIdx === currentNavigationIndex
                    ? 'text-zinc-100'
                    : 'text-zinc-400 group-hover:text-zinc-300',
                  'transition text-sm font-display leading-6',
                ]"
              >
                {{ nav.label }}
              </p>
            </div>
          </div>
        </NuxtLink>
      </ul>
    </div>
    <div class="grow overflow-y-scroll">
      <NuxtPage />
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Game } from "@prisma/client";
import { invoke } from "@tauri-apps/api/core";
import type { NavigationItem } from "~/types";

const rawGames = await invoke<string>("fetch_library");
const games: Array<Game> = JSON.parse(rawGames);
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
