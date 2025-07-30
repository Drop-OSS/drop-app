<template>
  <div class="p-8">
    <!-- Page Header -->
    <div class="mb-8">
      <h2 class="text-3xl font-bold font-display text-zinc-100 mb-2">
        Downloads
      </h2>
      <p class="text-lg text-zinc-400">
        {{ queue.queue.length }} items in queue
      </p>
    </div>

    <!-- Download Stats -->
    <div class="mb-8">
      <div class="bg-zinc-800 rounded-xl p-6">
        <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
          <div class="text-center">
            <div class="text-2xl font-bold text-blue-400 mb-1">
              {{ formatSpeed(stats.speed) }}/s
            </div>
            <div class="text-sm text-zinc-400">Download Speed</div>
          </div>
          <div class="text-center">
            <div class="text-2xl font-bold text-green-400 mb-1">
              {{ formatTime(stats.time) }}
            </div>
            <div class="text-sm text-zinc-400">Time Remaining</div>
          </div>
          <div class="text-center">
            <div class="text-2xl font-bold text-zinc-300 mb-1">
              {{ queue.queue.length }}
            </div>
            <div class="text-sm text-zinc-400">Items in Queue</div>
          </div>
        </div>
      </div>
    </div>

    <!-- Queue Items -->
    <div v-if="queue.queue.length > 0" class="space-y-4">
      <BigPictureQueueItem
        v-for="(item, index) in queue.queue"
        :key="item.meta.id"
        :item="item"
        :index="index"
        @cancel="cancelDownload"
        @move="moveDownload"
      />
    </div>

    <!-- Empty State -->
    <div v-else class="flex flex-col items-center justify-center py-16">
      <div class="text-center">
        <QueueListIcon class="h-24 w-24 text-zinc-600 mx-auto mb-6" />
        <h3 class="text-2xl font-semibold text-zinc-300 mb-2">
          No downloads in queue
        </h3>
        <p class="text-zinc-500 mb-6">
          Visit your library to install games
        </p>
        <NuxtLink
          to="/big-picture/library"
          class="inline-flex items-center px-6 py-3 bg-blue-600 hover:bg-blue-500 text-white font-semibold rounded-lg transition-all duration-200 transform hover:scale-105"
        >
          <BookOpenIcon class="h-5 w-5 mr-2" />
          View Library
        </NuxtLink>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { QueueListIcon, BookOpenIcon } from "@heroicons/vue/24/outline";
import { invoke } from "@tauri-apps/api/core";
import { type DownloadableMetadata } from "~/types";
import { useQueueState, useStatsState } from "~/composables/downloads";

definePageMeta({
  layout: "big-picture"
});

const queue = useQueueState();
const stats = useStatsState();

const formatSpeed = (bytesPerSecond: number): string => {
  const units = ["KB", "MB", "GB", "TB", "PB"];
  let value = bytesPerSecond;
  let unitIndex = 0;
  const scalar = 1000;

  while (value >= scalar && unitIndex < units.length - 1) {
    value /= scalar;
    unitIndex++;
  }

  return `${value.toFixed(1)} ${units[unitIndex]}`;
};

const formatTime = (seconds: number): string => {
  if (seconds < 60) {
    return `${Math.round(seconds)}s`;
  }
  
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) {
    return `${minutes}m ${Math.round(seconds % 60)}s`;
  }
  
  const hours = Math.floor(minutes / 60);
  return `${hours}h ${minutes % 60}m`;
};

const cancelDownload = async (meta: DownloadableMetadata) => {
  try {
    await invoke("cancel_game", { meta });
  } catch (error) {
    console.error("Failed to cancel download:", error);
  }
};

const moveDownload = async (oldIndex: number, newIndex: number) => {
  try {
    await invoke("move_download_in_queue", { oldIndex, newIndex });
  } catch (error) {
    console.error("Failed to move download:", error);
  }
};
</script> 