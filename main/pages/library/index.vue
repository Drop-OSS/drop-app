<template>
  <div v-if="libraryView === 'grid'" class="h-full overflow-y-auto p-6">
    <div
      v-if="loading"
      class="h-full flex items-center justify-center text-zinc-100"
    >
      <div role="status">
        <svg
          aria-hidden="true"
          class="w-6 h-6 text-transparent animate-spin fill-zinc-600"
          viewBox="0 0 100 101"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
        >
          <path
            d="M100 50.5908C100 78.2051 77.6142 100.591 50 100.591C22.3858 100.591 0 78.2051 0 50.5908C0 22.9766 22.3858 0.59082 50 0.59082C77.6142 0.59082 100 22.9766 100 50.5908ZM9.08144 50.5908C9.08144 73.1895 27.4013 91.5094 50 91.5094C72.5987 91.5094 90.9186 73.1895 90.9186 50.5908C90.9186 27.9921 72.5987 9.67226 50 9.67226C27.4013 9.67226 9.08144 27.9921 9.08144 50.5908Z"
            fill="currentColor"
          />
          <path
            d="M93.9676 39.0409C96.393 38.4038 97.8624 35.9116 97.0079 33.5539C95.2932 28.8227 92.871 24.3692 89.8167 20.348C85.8452 15.1192 80.8826 10.7238 75.2124 7.41289C69.5422 4.10194 63.2754 1.94025 56.7698 1.05124C51.7666 0.367541 46.6976 0.446843 41.7345 1.27873C39.2613 1.69328 37.813 4.19778 38.4501 6.62326C39.0873 9.04874 41.5694 10.4717 44.0505 10.1071C47.8511 9.54855 51.7191 9.52689 55.5402 10.0491C60.8642 10.7766 65.9928 12.5457 70.6331 15.2552C75.2735 17.9648 79.3347 21.5619 82.5849 25.841C84.9175 28.9121 86.7997 32.2913 88.1811 35.8758C89.083 38.2158 91.5421 39.6781 93.9676 39.0409Z"
            fill="currentFill"
          />
        </svg>
        <span class="sr-only">Loading...</span>
      </div>
    </div>
    <div
      v-else
      class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 2xl:grid-cols-7 gap-4"
    >
      <template v-for="collection in filteredNavigation" :key="collection.id">
        <NuxtLink
          v-for="item in collection.items"
          :key="item.id"
          :href="item.route"
          :class="[
            'group relative aspect-[3/4] transition-all duration-200 rounded-xl overflow-hidden',
            'hover:scale-[1.02] active:scale-[0.98]',
            'focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:ring-offset-zinc-900',
            currentNavigation == item.id
              ? 'ring-2 ring-blue-500 shadow-lg shadow-blue-500/20'
              : 'ring-1 ring-zinc-800/50',
          ]"
        >
          <!-- Blurred Background Image -->
          <div class="absolute inset-0 z-0">
            <!-- Cover Image (if available) -->
            <img
              v-if="covers[item.id]"
              :src="covers[item.id]"
              :alt="item.label"
              class="w-full h-full object-cover blur-[2px] scale-105 transition-transform duration-300 group-hover:scale-110"
            />
            <!-- Fallback to Icon -->
            <div
              v-else
              class="w-full h-full flex items-center justify-center bg-gradient-to-br from-zinc-800 to-zinc-900"
            >
              <img
                class="w-2/3 h-2/3 object-contain blur-[2px] scale-105 transition-transform duration-300 group-hover:scale-110"
                :src="icons[item.id]"
                :alt="item.label"
              />
            </div>
            <!-- Gradient Overlays -->
            <div
              class="absolute inset-0 bg-gradient-to-t from-zinc-900 via-zinc-900/80 to-transparent opacity-90 transition-opacity duration-200 group-hover:opacity-70"
            />
            <div
              class="absolute inset-0 bg-gradient-to-r from-zinc-900/95 via-zinc-900/80 to-transparent opacity-90 transition-opacity duration-200 group-hover:opacity-60"
            />
          </div>

          <!-- Content Overlay -->
          <div class="relative z-10 flex flex-col h-full p-4 justify-end">
            <!-- Game Title -->
            <h3
              :class="[
                'text-base font-display font-bold text-zinc-100 drop-shadow-lg mb-2 line-clamp-2',
                'transition-colors duration-200',
                currentNavigation == item.id
                  ? 'text-zinc-100'
                  : item.isInstalled.value
                  ? 'text-zinc-100 group-hover:text-white'
                  : 'text-zinc-300 group-hover:text-zinc-100',
              ]"
            >
              {{ item.label }}
            </h3>

            <!-- Status Badge -->
            <div class="flex items-center">
              <span
                :class="[
                  'inline-flex items-center px-2 py-1 rounded-md text-[10px] font-bold uppercase font-display',
                  'backdrop-blur-sm border shadow-lg',
                  gameStatusTextStyle[games[item.id].status.value.type],
                  currentNavigation == item.id
                    ? 'bg-zinc-800/50 border-zinc-700/50 text-white'
                    : 'bg-zinc-900/50 border-zinc-800/50',
                ]"
              >
                {{ gameStatusText[games[item.id].status.value.type] }}
              </span>
            </div>
          </div>

          <!-- Selected Indicator -->
          <div
            v-if="currentNavigation == item.id"
            class="absolute top-3 right-3 z-20 w-2.5 h-2.5 rounded-full bg-blue-500 shadow-lg shadow-blue-500/50 ring-2 ring-blue-500/30"
          />
        </NuxtLink>
      </template>
    </div>
  </div>
  <div v-else class="h-full flex flex-col items-center justify-center">
    <div class="text-center">
      <div class="flex flex-col items-center gap-y-4">
        <div class="p-4 rounded-xl bg-zinc-700/50 backdrop-blur-sm">
          <RocketLaunchIcon class="size-12 text-zinc-400" />
        </div>
        <div>
          <h3 class="text-xl font-display font-semibold text-zinc-100">
            Select a game
          </h3>
          <p class="mt-1 text-sm text-zinc-400">
            Choose a game from your library to view details
          </p>
        </div>
      </div>
    </div>
  </div>
</template>
<script setup lang="ts">
import { RocketLaunchIcon } from "@heroicons/vue/24/outline";
import { invoke } from "@tauri-apps/api/core";
import {
  GameStatusEnum,
  type Collection,
  type Game,
  type GameStatus,
} from "~/types";
import { listen } from "@tauri-apps/api/event";

const { libraryView } = useLibraryView();

const route = useRoute();
const currentNavigation = computed(() => {
  return route.path.slice("/library/".length);
});

// Style information
const gameStatusTextStyle: { [key in GameStatusEnum]: string } = {
  [GameStatusEnum.Installed]: "text-green-500",
  [GameStatusEnum.Downloading]: "text-zinc-400",
  [GameStatusEnum.Validating]: "text-blue-300",
  [GameStatusEnum.Running]: "text-green-500",
  [GameStatusEnum.Remote]: "text-zinc-700",
  [GameStatusEnum.Queued]: "text-zinc-400",
  [GameStatusEnum.Updating]: "text-zinc-400",
  [GameStatusEnum.Uninstalling]: "text-zinc-100",
  [GameStatusEnum.SetupRequired]: "text-yellow-500",
  [GameStatusEnum.PartiallyInstalled]: "text-gray-400",
};
const gameStatusText: { [key in GameStatusEnum]: string } = {
  [GameStatusEnum.Remote]: "Not installed",
  [GameStatusEnum.Queued]: "Queued",
  [GameStatusEnum.Downloading]: "Downloading...",
  [GameStatusEnum.Validating]: "Validating...",
  [GameStatusEnum.Installed]: "Installed",
  [GameStatusEnum.Updating]: "Updating...",
  [GameStatusEnum.Uninstalling]: "Uninstalling...",
  [GameStatusEnum.SetupRequired]: "Setup required",
  [GameStatusEnum.Running]: "Running",
  [GameStatusEnum.PartiallyInstalled]: "Partially installed",
};

const loading = ref(false);
const games: {
  [key: string]: { game: Game; status: Ref<GameStatus, GameStatus> };
} = {};
const icons: { [key: string]: string } = {};
const covers: { [key: string]: string } = {};
const collections: Ref<Collection[]> = ref([]);

async function calculateGames(clearAll = false, forceRefresh = false) {
  if (clearAll) {
    collections.value = [];
    loading.value = true;
  }
  const newGames = await invoke<Game[]>("fetch_library", {
    hardRefresh: forceRefresh,
  });
  const otherCollections = await invoke<Collection[]>("fetch_collections", {
    hardRefresh: forceRefresh,
  });
  const allGames = [
    ...newGames,
    ...otherCollections
      .map((e) => e.entries)
      .flat()
      .map((e) => e.game),
  ].filter((v, i, a) => a.indexOf(v) === i);

  for (const game of allGames) {
    if (games[game.id]) continue;
    games[game.id] = await useGame(game.id);
  }
  for (const game of allGames) {
    if (icons[game.id]) continue;
    icons[game.id] = await useObject(game.mIconObjectId);
  }
  for (const game of allGames) {
    if (covers[game.id]) continue;
    covers[game.id] = await useObject(game.mCoverObjectId);
  }

  const libraryCollection = {
    id: "library",
    name: "Library",
    isDefault: true,
    entries: newGames.map((e) => ({ gameId: e.id, game: e })),
  } satisfies Collection;

  loading.value = false;
  collections.value = [libraryCollection, ...otherCollections];
}

// Wait up to 300 ms for the library to load
await new Promise<void>((r) => {
  let hasResolved = false;
  const resolveFunc = () => {
    if (!hasResolved) r();
    hasResolved = true;
  };
  calculateGames(true).then(resolveFunc);
  setTimeout(resolveFunc, 300);
});

const navigation = computed(() =>
  collections.value.map((collection) => {
    const items = collection.entries.map(({ game }) => {
      const status = games[game.id].status;

      const isInstalled = computed(
        () => status.value.type != GameStatusEnum.Remote
      );

      const item = {
        label: game.mName,
        route: `/library/${game.id}`,
        prefix: `/library/${game.id}`,
        isInstalled,
        id: game.id,
      };
      return item;
    });

    return {
      id: collection.id,
      name: collection.name,
      deft: collection.isDefault,
      items,
    };
  })
);

const filteredNavigation = computed(() => {
  return navigation.value;
});

listen("update_library", async () => {
  await calculateGames();
});
</script>