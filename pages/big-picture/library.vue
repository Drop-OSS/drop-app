<template>
  <div class="p-8">
    <!-- Page Header -->
    <div class="mb-8">
      <h2 class="text-3xl font-bold font-display text-zinc-100 mb-2">
        Your Library
      </h2>
      <p class="text-lg text-zinc-400">
        {{ games.length }} games in your collection
      </p>
    </div>

    <!-- Games Grid -->
    <div v-if="games.length > 0" class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
      <BigPictureGameTile
        v-for="game in games"
        :key="game.id"
        :game="game"
        @click="navigateToGame(game.id)"
      />
    </div>

    <!-- Empty State -->
    <div v-else class="flex flex-col items-center justify-center py-16">
      <div class="text-center">
        <BookOpenIcon class="h-24 w-24 text-zinc-600 mx-auto mb-6" />
        <h3 class="text-2xl font-semibold text-zinc-300 mb-2">
          No games in your library
        </h3>
        <p class="text-zinc-500 mb-6">
          Visit the store to discover and install games
        </p>
        <NuxtLink
          to="/big-picture/store"
          class="inline-flex items-center px-6 py-3 bg-blue-600 hover:bg-blue-500 text-white font-semibold rounded-lg transition-all duration-200 transform hover:scale-105"
        >
          <BuildingStorefrontIcon class="h-5 w-5 mr-2" />
          Visit Store
        </NuxtLink>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { BookOpenIcon, BuildingStorefrontIcon } from "@heroicons/vue/24/outline";
import { invoke } from "@tauri-apps/api/core";
import BigPictureGameTile from "~/components/BigPictureGameTile.vue";

definePageMeta({
  layout: "big-picture"
});

const router = useRouter();

// Fetch games from backend
const games = ref<Game[]>([]);

onMounted(async () => {
  try {
    const libraryData = await invoke("fetch_library");
    games.value = libraryData as Game[];
  } catch (error) {
    console.error("Failed to fetch library:", error);
  }
});

const navigateToGame = (gameId: string) => {
  router.push(`/big-picture/library/${gameId}`);
};
</script> 