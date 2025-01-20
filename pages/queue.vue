<template>
  <div class="bg-zinc-950 p-4 min-h-full space-y-4">
    <div
      class="h-16 overflow-hidden relative rounded-xl flex flex-row border border-zinc-900"
    >
      <div
        class="bg-zinc-900 z-10 w-32 flex flex-col gap-x-2 text-blue-400 font-display items-left justify-center pl-2"
      >
        <span class="font-semibold">{{ formatKilobytes(stats.speed) }}/s</span>
        <span v-if="stats.time > 0" class="text-sm"
          >{{ formatTime(stats.time) }} left</span
        >
      </div>
      <div class="absolute inset-0 h-full flex flex-row items-end justify-end">
        <div
          v-for="bar in speedHistory"
          :style="{ height: `${(bar / speedMax) * 100}%` }"
          class="w-[8px] bg-blue-600/40"
        />
      </div>
    </div>
    <draggable v-model="queue.queue" @end="onEnd">
      <template #item="{ element }: { element: (typeof queue.value.queue)[0] }">
        <li
          v-if="games[element.meta.id]"
          :key="element.meta.id"
          class="mb-4 bg-zinc-900 rounded-lg flex flex-row justify-between gap-x-6 py-5 px-4"
        >
          <div class="w-full flex items-center max-w-md gap-x-4 relative">
            <img
              class="size-24 flex-none bg-zinc-800 object-cover rounded"
              :src="games[element.meta.id].cover"
              alt=""
            />
            <div class="min-w-0 flex-auto">
              <p class="text-xl font-semibold text-zinc-100">
                <NuxtLink :href="`/library/${element.meta.id}`" class="">
                  <span class="absolute inset-x-0 -top-px bottom-0" />
                  {{ games[element.meta.id].game.mName }}
                </NuxtLink>
              </p>
              <p class="mt-1 flex text-xs/5 text-gray-500">
                {{ games[element.meta.id].game.mShortDescription }}
              </p>
            </div>
          </div>
          <div class="flex shrink-0 items-center gap-x-4">
            <div class="hidden sm:flex sm:flex-col sm:items-end">
              <p class="text-md text-zinc-500 uppercase font-display font-bold">
                {{ element.status }}
              </p>
              <div
                v-if="element.progress"
                class="mt-1 w-96 bg-zinc-800 rounded-lg overflow-hidden"
              >
                <div
                  class="h-2 bg-blue-600"
                  :style="{ width: `${element.progress * 100}%` }"
                />
              </div>
              <span
                class="mt-2 inline-flex items-center gap-x-1 text-zinc-400 text-sm font-display"
                ><span class="text-zinc-300">{{
                  formatKilobytes(element.current / 1000)
                }}</span>
                /
                <span class="">{{ formatKilobytes(element.max / 1000) }}</span
                ><ServerIcon class="size-5"
              /></span>
            </div>
            <button @click="() => cancelGame(element.meta)" class="group">
              <XMarkIcon
                class="transition size-8 flex-none text-zinc-600 group-hover:text-zinc-300"
                aria-hidden="true"
              />
            </button>
          </div>
        </li>
        <p v-else>Loading...</p>
      </template>
    </draggable>
    <div
      class="text-zinc-600 uppercase font-semibold font-display w-full text-center"
      v-if="queue.queue.length == 0"
    >
      No items in the queue
    </div>
  </div>
</template>

<script setup lang="ts">
import { ServerIcon, XMarkIcon } from "@heroicons/vue/20/solid";
import { invoke } from "@tauri-apps/api/core";
import type { DownloadableMetadata, Game, GameStatus } from "~/types";

const windowWidth = ref(window.innerWidth);
window.addEventListener("resize", (event) => {
  windowWidth.value = window.innerWidth;
});

const queue = useQueueState();
const stats = useStatsState();
const speedHistory = useState<Array<number>>(() => []);
const speedHistoryMax = computed(() => windowWidth.value / 8);
const speedMax = computed(
  () => speedHistory.value.reduce((a, b) => (a > b ? a : b)) * 1.3
);
const previousGameId = ref<string | undefined>();

const games: Ref<{
  [key: string]: { game: Game; status: Ref<GameStatus>; cover: string };
}> = ref({});

function resetHistoryGraph() {
  speedHistory.value = [];
  stats.value = { time: 0, speed: 0 };
}
function checkReset(v: QueueState) {
  const currentGame = v.queue.at(0)?.meta.id;
  // If we're finished
  if (!currentGame && previousGameId.value) {
    previousGameId.value = undefined;
    resetHistoryGraph();
    return;
  }
  // If we don't have a game
  if (!currentGame) return;
  // If we started a new download
  if (currentGame && !previousGameId.value) {
    previousGameId.value = currentGame;
    resetHistoryGraph();
    return;
  }
  // If it's a different game now
  if (currentGame != previousGameId.value) {
    previousGameId.value = currentGame;
    resetHistoryGraph();
    return;
  }
}
watch(queue, (v) => {
  loadGamesForQueue(v);
  checkReset(v);
});

watch(stats, (v) => {
  const newLength = speedHistory.value.push(v.speed);
  if (newLength > speedHistoryMax.value) {
    speedHistory.value.splice(0, 1);
  }
  checkReset(queue.value);
});

function loadGamesForQueue(v: typeof queue.value) {
  for (const {
    meta: { id },
  } of v.queue) {
    if (games.value[id]) return;
    (async () => {
      const gameData = await useGame(id);
      const cover = await useObject(gameData.game.mCoverId);
      games.value[id] = { ...gameData, cover };
    })();
  }
}

loadGamesForQueue(queue.value);

async function onEnd(event: { oldIndex: number; newIndex: number }) {
  await invoke("move_game_in_queue", {
    oldIndex: event.oldIndex,
    newIndex: event.newIndex,
  });
}

async function cancelGame(meta: DownloadableMetadata) {
  await invoke("cancel_game", { meta });
}

function formatKilobytes(bytes: number): string {
  const units = ["KB", "MB", "GB", "TB", "PB"];
  let value = bytes;
  let unitIndex = 0;
  const scalar = 1000;

  while (value >= scalar && unitIndex < units.length - 1) {
    value /= scalar;
    unitIndex++;
  }

  return `${value.toFixed(1)} ${units[unitIndex]}`;
}

function formatTime(seconds: number): string {
  if (seconds < 60) {
    return `${Math.round(seconds)}s`;
  }

  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) {
    return `${minutes}m ${Math.round(seconds % 60)}s`;
  }

  const hours = Math.floor(minutes / 60);
  return `${hours}h ${minutes % 60}m`;
}
</script>
