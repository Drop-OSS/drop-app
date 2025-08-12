<template>
  <div class="flex flex-col h-full">
    <div class="mb-3 inline-flex gap-x-2">
      <div
        class="relative transition-transform duration-300 hover:scale-105 active:scale-95"
      >
        <div
          class="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-3"
        >
          <MagnifyingGlassIcon
            class="h-5 w-5 text-zinc-400"
            aria-hidden="true"
          />
        </div>
        <input
          type="text"
          v-model="searchQuery"
          class="block w-full rounded-lg border-0 bg-zinc-800/50 py-2 pl-10 pr-3 text-zinc-100 placeholder:text-zinc-500 focus:bg-zinc-800 focus:ring-2 focus:ring-inset focus:ring-blue-500 sm:text-sm sm:leading-6"
          placeholder="Search library..."
        />
      </div>
      <button
        @click="() => calculateGames(true)"
        class="p-1 flex items-center justify-center transition-transform duration-300 size-10 hover:scale-110 active:scale-90 rounded-lg bg-zinc-800/50 text-zinc-100"
      >
        <ArrowPathIcon class="size-4" />
      </button>
    </div>

    <TransitionGroup name="list" tag="ul" class="flex flex-col gap-y-1.5">
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
          <div
            class="flex-none transition-transform duration-300 hover:-rotate-2"
          >
            <img
              class="size-8 object-cover bg-zinc-900 rounded-lg transition-all duration-300 shadow-sm"
              :src="icons[nav.id]"
              alt=""
            />
          </div>
          <div class="flex flex-col flex-1">
            <p
              class="truncate text-xs font-display leading-5 flex-1 font-semibold"
            >
              {{ nav.label }}
            </p>
            <p
              class="text-xs font-medium"
              :class="[gameStatusTextStyle[games[nav.id].status.value.type]]"
            >
              {{ gameStatusText[games[nav.id].status.value.type] }}
            </p>
          </div>
        </div>
      </NuxtLink>
    </TransitionGroup>
    <div
      v-if="loading"
      class="h-full grow flex p-8 justify-center text-zinc-100"
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
  </div>
</template>

<script setup lang="ts">
import { ArrowPathIcon, MagnifyingGlassIcon } from "@heroicons/vue/20/solid";
import { invoke } from "@tauri-apps/api/core";
import { GameStatusEnum, type Game, type GameStatus } from "~/types";
import { TransitionGroup } from "vue";
import { listen } from "@tauri-apps/api/event";

// Style information
const gameStatusTextStyle: { [key in GameStatusEnum]: string } = {
  [GameStatusEnum.Installed]: "text-green-500",
  [GameStatusEnum.Downloading]: "text-zinc-400",
  [GameStatusEnum.Validating]: "text-blue-300",
  [GameStatusEnum.Running]: "text-green-500",
  [GameStatusEnum.Remote]: "text-zinc-500",
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

const router = useRouter();

const searchQuery = ref("");

const loading = ref(false);
const games: {
  [key: string]: { game: Game; status: Ref<GameStatus, GameStatus> };
} = {};
const icons: { [key: string]: string } = {};

const rawGames: Ref<Game[], Game[]> = ref([]);

async function calculateGames(clearAll = false) {
  if (clearAll) {
    rawGames.value = [];
    loading.value = true;
  }
  // If we update immediately, the navigation gets re-rendered before we
  // add all the necessary state, and it freaks tf out
  const newGames = await invoke<typeof rawGames.value>("fetch_library");
  for (const game of newGames) {
    if (games[game.id]) continue;
    games[game.id] = await useGame(game.id);
  }
  for (const game of newGames) {
    if (icons[game.id]) continue;
    icons[game.id] = await useObject(game.mIconObjectId);
  }
  loading.value = false;
  rawGames.value = newGames;
}

calculateGames(true);

const navigation = computed(() =>
  rawGames.value.map((game) => {
    const status = games[game.id].status;

    const isInstalled = computed(
      () =>
        status.value.type != GameStatusEnum.Remote
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
const { currentNavigation, recalculateNavigation } = useCurrentNavigationIndex(
  navigation.value
);

const filteredNavigation = computed(() => {
  if (!searchQuery.value)
    return navigation.value.map((e, i) => ({ ...e, index: i }));
  const query = searchQuery.value.toLowerCase();
  return navigation.value
    .filter((nav) => nav.label.toLowerCase().includes(query))
    .map((e, i) => ({ ...e, index: i }));
});

listen("update_library", async (event) => {
  console.log("Updating library");
  let oldNavigation = navigation.value[currentNavigation.value];
  await calculateGames();
  recalculateNavigation();
  if (oldNavigation !== navigation.value[currentNavigation.value]) {
    console.log("Triggered");
    router.push("/library");
  }
});
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
