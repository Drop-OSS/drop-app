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
          nav.index === currentNavigation
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
              :src="icons[nav.id]"
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
                games[nav.id].status.value.type === GameStatusEnum.Installed ? 'text-green-500' : 
                games[nav.id].status.value.type === GameStatusEnum.Downloading ? 'text-blue-500' :
                games[nav.id].status.value.type === GameStatusEnum.Running ? 'text-green-500' :
                'text-zinc-500'
              ]"
            >
              {{ 
                games[nav.id].status.value.type === GameStatusEnum.Downloading ? 'Downloading' :
                games[nav.id].status.value.type === GameStatusEnum.Running ? 'Running' :
                games[nav.id].status.value.type ? 'Installed' : 'Not Installed'
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
import { GameStatusEnum, type Game, type GameStatus } from "~/types";
import { TransitionGroup } from 'vue';
import { listen } from '@tauri-apps/api/event';

const router = useRouter();

const searchQuery = ref('');


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


const filteredNavigation = computed(() => {
  if (!searchQuery.value) return navigation.value.map((e, i) => ({...e, index: i}));
  const query = searchQuery.value.toLowerCase();
  return navigation.value.filter(nav => 
    nav.label.toLowerCase().includes(query)
  ).map((e, i) => ({...e, index: i}));
});

listen("update_library", async (event) => {
  console.log("Updating library");
  let oldNavigation = navigation.value[currentNavigation.value];
  await calculateGames()
  recalculateNavigation();
  if (oldNavigation !== navigation.value[currentNavigation.value]) {
    console.log("Triggered")
    router.push("/library")
  }
})

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