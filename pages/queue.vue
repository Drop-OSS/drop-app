<template>
  <draggable v-model="queue.queue" @end="onEnd">
    <template #item="{ element }: { element: (typeof queue.value.queue)[0] }">
      <div class="text-white">
        {{ element.id }}
      </div>
    </template>
  </draggable>
  {{ current }}
  {{ rest }}
</template>

<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";

const queue = useQueueState();

const current = computed(() => queue.value.queue.at(0));
const rest = computed(() => queue.value.queue.slice(1));

async function onEnd(event: { oldIndex: number; newIndex: number }) {
  await invoke("move_game_in_queue", { oldIndex: event.oldIndex, newIndex: event.newIndex });
}
</script>
