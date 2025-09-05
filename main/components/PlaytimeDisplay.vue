<template>
  <div v-if="stats" class="flex flex-col gap-1">
    <!-- Main playtime display -->
    <div class="flex items-center gap-2">
      <ClockIcon class="w-5 h-5 text-zinc-400" />
      <span class="text-base text-zinc-300 font-medium">
        {{ formatPlaytime(stats.totalPlaytimeSeconds) }} played
      </span>
      <span v-if="isActive && showActiveIndicator" class="text-sm text-green-400 font-medium">
        • Playing
      </span>
    </div>
    
    <!-- Additional details when expanded -->
    <div v-if="showDetails" class="text-xs text-zinc-400 space-y-1 ml-7">
      <div>{{ stats.sessionCount }} session{{ stats.sessionCount !== 1 ? 's' : '' }}</div>
      <div v-if="stats.sessionCount > 0">
        Avg: {{ formatPlaytime(stats.averageSessionLength) }} per session
      </div>
      <div>Last played {{ formatRelativeTime(stats.lastPlayed) }}</div>
      <div v-if="stats.currentSessionDuration">
        Current session: {{ formatPlaytime(stats.currentSessionDuration) }}
      </div>
    </div>
  </div>
  
  <!-- No playtime data -->
  <div v-else-if="showWhenEmpty" class="flex items-center gap-2 text-zinc-500">
    <ClockIcon class="w-5 h-5" />
    <span class="text-base">Never played</span>
  </div>
</template>

<script setup lang="ts">
import { ClockIcon } from "@heroicons/vue/20/solid";
import type { GamePlaytimeStats } from "~/types";

interface Props {
  stats: GamePlaytimeStats | null;
  isActive?: boolean;
  showDetails?: boolean;
  showWhenEmpty?: boolean;
  showActiveIndicator?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  isActive: false,
  showDetails: false,
  showWhenEmpty: true,
  showActiveIndicator: true,
});

const { formatPlaytime, formatRelativeTime } = usePlaytime();
</script>
