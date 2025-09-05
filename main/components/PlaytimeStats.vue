<template>
  <div class="bg-zinc-800/50 rounded-lg p-4 space-y-4">
    <div class="flex items-center gap-2">
      <ChartBarIcon class="w-5 h-5 text-zinc-400" />
      <h3 class="text-lg font-semibold text-zinc-100">Playtime Statistics</h3>
    </div>
    
    <div v-if="stats" class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <!-- Total Playtime -->
      <div class="bg-zinc-700/50 rounded-lg p-3">
        <div class="flex items-center gap-2 mb-2">
          <ClockIcon class="w-4 h-4 text-blue-400" />
          <span class="text-sm font-medium text-zinc-300">Total Playtime</span>
        </div>
        <div class="text-2xl font-bold text-zinc-100">
          {{ formatDetailedPlaytime(stats.totalPlaytimeSeconds) }}
        </div>
        <div v-if="stats.currentSessionDuration" class="text-xs text-green-400 mt-1">
          +{{ formatPlaytime(stats.currentSessionDuration) }} this session
        </div>
      </div>
      
      <!-- Sessions -->
      <div class="bg-zinc-700/50 rounded-lg p-3">
        <div class="flex items-center gap-2 mb-2">
          <PlayIcon class="w-4 h-4 text-green-400" />
          <span class="text-sm font-medium text-zinc-300">Sessions</span>
        </div>
        <div class="text-2xl font-bold text-zinc-100">
          {{ stats.sessionCount }}
        </div>
        <div class="text-xs text-zinc-400 mt-1">
          Avg: {{ formatPlaytime(stats.averageSessionLength) }}
        </div>
      </div>
      
      <!-- First Played -->
      <div class="bg-zinc-700/50 rounded-lg p-3">
        <div class="flex items-center gap-2 mb-2">
          <CalendarIcon class="w-4 h-4 text-purple-400" />
          <span class="text-sm font-medium text-zinc-300">First Played</span>
        </div>
        <div class="text-lg font-semibold text-zinc-100">
          {{ formatDate(stats.firstPlayed) }}
        </div>
        <div class="text-xs text-zinc-400 mt-1">
          {{ formatRelativeTime(stats.firstPlayed) }}
        </div>
      </div>
      
      <!-- Last Played -->
      <div class="bg-zinc-700/50 rounded-lg p-3">
        <div class="flex items-center gap-2 mb-2">
          <ClockIcon class="w-4 h-4 text-orange-400" />
          <span class="text-sm font-medium text-zinc-300">Last Played</span>
        </div>
        <div class="text-lg font-semibold text-zinc-100">
          {{ formatDate(stats.lastPlayed) }}
        </div>
        <div class="text-xs text-zinc-400 mt-1">
          {{ formatRelativeTime(stats.lastPlayed) }}
        </div>
      </div>
    </div>
    
    <!-- No stats available -->
    <div v-else class="text-center py-8">
      <ClockIcon class="w-12 h-12 text-zinc-600 mx-auto mb-3" />
      <p class="text-zinc-400">No playtime data available</p>
      <p class="text-sm text-zinc-500 mt-1">Statistics will appear after you start playing</p>
    </div>
    
    <!-- Current session indicator -->
    <div v-if="isActive && stats" class="border-t border-zinc-700 pt-3">
      <div class="flex items-center gap-2 text-green-400">
        <div class="w-2 h-2 bg-green-400 rounded-full animate-pulse"></div>
        <span class="text-sm font-medium">Currently playing</span>
        <span v-if="stats.currentSessionDuration" class="text-xs text-zinc-400">
          {{ formatPlaytime(stats.currentSessionDuration) }}
        </span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { 
  ChartBarIcon, 
  ClockIcon, 
  PlayIcon, 
  CalendarIcon 
} from "@heroicons/vue/20/solid";
import type { GamePlaytimeStats } from "~/types";

interface Props {
  stats: GamePlaytimeStats | null;
  isActive?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  isActive: false,
});

const { formatPlaytime, formatDetailedPlaytime, formatRelativeTime } = usePlaytime();

const formatDate = (timestamp: string): string => {
  const date = new Date(timestamp);
  return date.toLocaleDateString(undefined, { 
    year: 'numeric', 
    month: 'short', 
    day: 'numeric' 
  });
};
</script>
