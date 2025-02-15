<template>
  <div>
    <div class="relative mb-3 transition-transform duration-300 hover:scale-105 active:scale-95">
      <div class="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-3">
        <MagnifyingGlassIcon class="h-5 w-5 text-zinc-400" aria-hidden="true" />
      </div>
      <input
        type="text"
        v-model="searchQuery"
        class="block w-full rounded-lg border-0 bg-zinc-800/50 py-2 pl-10 pr-3 text-zinc-100 placeholder:text-zinc-500 focus:bg-zinc-800 focus:ring-2 focus:ring-inset focus:ring-blue-500 sm:text-sm sm:leading-6"
        placeholder="Search library..."
      />
    </div>

    <TransitionGroup 
      name="list" 
      tag="ul" 
      class="flex flex-col gap-y-1.5"
    >
      <NuxtLink
        v-for="nav in filteredNavigation"
        :key="nav.id"
        :class="[
          'transition-all duration-300 rounded-lg flex items-center py-2 px-3 hover:scale-105 active:scale-95 hover:shadow-lg hover:shadow-zinc-950/50',
          nav.index === currentNavigationIndex
            ? 'bg-zinc-800 text-zinc-100 shadow-md shadow-zinc-950/20'
            : nav.isInstalled.value
            ? 'text-zinc-300 hover:bg-zinc-800/90 hover:text-zinc-200'
            : 'text-zinc-500 hover:bg-zinc-800/70 hover:text-zinc-300',
        ]"
        :href="nav.route"
      >
        <div class="flex items-center w-full gap-x-3">
          <div class="flex-none transition-transform duration-300 hover:-rotate-2">
            <img
              class="size-8 object-cover bg-zinc-900 rounded-lg transition-all duration-300 shadow-sm"
              :src="nav.icon"
              alt=""
            />
          </div>
          <div class="flex flex-col flex-1">
            <p class="truncate text-xs font-display leading-5 flex-1 font-semibold">
              {{ nav.label }}
            </p>
            <p 
              class="text-[11px] font-medium"
              :class="[
                nav.status.value.type === GameStatusEnum.Installed ? 'text-green-500' : 
                nav.status.value.type === GameStatusEnum.Downloading ? 'text-blue-500' :
                nav.status.value.type === GameStatusEnum.Running ? 'text-green-500' :
                'text-zinc-500'
              ]"
            >
              {{ 
                nav.status.value.type === GameStatusEnum.Downloading ? 'Downloading' :
                nav.status.value.type === GameStatusEnum.Running ? 'Running' :
                nav.isInstalled.value ? 'Installed' : 'Not Installed'
              }}
            </p>
          </div>
        </div>
      </NuxtLink>
    </TransitionGroup>
  </div>
</template>

<script setup lang="ts">
import { MagnifyingGlassIcon } from '@heroicons/vue/20/solid';
import { invoke } from "@tauri-apps/api/core";
import { GameStatusEnum, type Game, type NavigationItem } from "~/types";
import { TransitionGroup } from 'vue';

const searchQuery = ref('');

async function calculateGames(): Promise<Game[]> {
  try {
    return await invoke("fetch_library");
  }
  catch(e) {
    return new Array();
  }
}

const rawGames: Array<Game> = await calculateGames();
const games = await Promise.all(rawGames.map((e) => useGame(e.id)));
const icons = await Promise.all(
  games.map(({ game, status }) => useObject(game.mIconId))
);

const navigation = games.map(({ game, status }, index) => {
  const isInstalled = computed(
    () =>
      status.value.type === GameStatusEnum.Installed ||
      status.value.type === GameStatusEnum.SetupRequired ||
      status.value.type === GameStatusEnum.Running
  );

  return {
    id: game.id,
    label: game.mName,
    route: `/library/${game.id}`,
    prefix: `/library/${game.id}`,
    isInstalled,
    status,
    icon: icons[index],
    index
  };
});

const filteredNavigation = computed(() => {
  if (!searchQuery.value) return navigation;
  const query = searchQuery.value.toLowerCase();
  return navigation.filter(nav => 
    nav.label.toLowerCase().includes(query)
  );
});

const currentNavigationIndex = useCurrentNavigationIndex(navigation);
</script>

<style scoped>
.list-move,
.list-enter-active,
.list-leave-active {
  transition: all 0.3s ease;
}

.list-enter-from,
.list-leave-to {
  opacity: 0;
  transform: translateX(-30px);
}

.list-leave-active {
  position: absolute;
}
</style> 