<template>
  <div class="bg-zinc-950 p-4 min-h-full">
    <draggable v-model="queue.queue" @end="onEnd">
      <template #item="{ element }: { element: (typeof queue.value.queue)[0] }">
        <li
          v-if="games[element.id]"
          :key="element.id"
          class="mb-4 bg-zinc-900 rounded-lg flex flex-row justify-between gap-x-6 py-5 px-4"
        >
          <div class="w-full flex items-center max-w-md gap-x-4 relative">
            <img
              class="size-24 flex-none bg-zinc-800 object-cover rounded"
              :src="games[element.id].cover"
              alt=""
            />
            <div class="min-w-0 flex-auto">
              <p class="text-xl font-semibold text-zinc-100">
                <NuxtLink :href="`/library/${element.id}`" class="">
                  <span class="absolute inset-x-0 -top-px bottom-0" />
                  {{ games[element.id].game.mName }}
                </NuxtLink>
              </p>
              <p class="mt-1 flex text-xs/5 text-gray-500">
                {{ games[element.id].game.mShortDescription }}
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
            </div>
            <button @click="() => cancelGame(element.id)" class="group">
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
import { XMarkIcon } from "@heroicons/vue/20/solid";
import { invoke } from "@tauri-apps/api/core";
import type { Game, GameStatus } from "~/types";

const queue = useQueueState();

const current = computed(() => queue.value.queue.at(0));
const rest = computed(() => queue.value.queue.slice(1));

const games: Ref<{
  [key: string]: { game: Game; status: Ref<GameStatus>; cover: string };
}> = ref({});

watch(queue, (v) => {
  loadGamesForQueue(v);
});

function loadGamesForQueue(v: typeof queue.value) {
  for (const { id } of v.queue) {
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

async function cancelGame(id: string) {
  await invoke("cancel_game", { gameId: id });
}
</script>
