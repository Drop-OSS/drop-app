<template>
  <div class="flex flex-col px-6 py-6 md:px-8 lg:px-10">
    <div class="max-w-2xl">
      <div class="flex items-center gap-x-3 mb-4">
        <NuxtLink
          to="/library"
          class="transition text-sm/6 font-semibold text-zinc-400 hover:text-zinc-100 inline-flex gap-x-2 items-center duration-200 hover:scale-105"
        >
          <ArrowLeftIcon class="h-4 w-4" aria-hidden="true" />
          Back to Collections
        </NuxtLink>
      </div>
      <h2 class="text-2xl font-bold font-display text-zinc-100">
        {{ collection?.name }}
      </h2>
      <p class="mt-2 text-zinc-400">
        {{ collection?.entries?.length || 0 }} games
      </p>
    </div>

    <!-- Games grid -->
    <div class="mt-8 grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4">
      <NuxtLink
        v-for="entry in collection?.entries"
        :key="entry.gameId"
        :to="`/library/${entry.gameId}`"
        class="group relative flex flex-col overflow-hidden rounded-lg bg-zinc-800/50 hover:bg-zinc-800 transition-all duration-200 hover:scale-105"
      >
        <div class="aspect-square w-full overflow-hidden">
          <img
            :src="entry.game.mBannerId || entry.game.mCoverId"
            :alt="entry.game.mName"
            class="h-full w-full object-cover"
          />
        </div>
        <div class="flex flex-1 flex-col justify-between p-4">
          <div class="flex-1">
            <p class="text-sm font-semibold text-zinc-100 line-clamp-2">
              {{ entry.game.mName }}
            </p>
          </div>
        </div>
      </NuxtLink>

      <!-- Empty state -->
      <div
        v-if="!collection?.entries?.length"
        class="col-span-full flex flex-col items-center justify-center py-12 text-center"
      >
        <p class="text-sm text-zinc-400">
          No games in this collection yet.
        </p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ArrowLeftIcon } from "@heroicons/vue/20/solid";

const route = useRoute();
const collection = ref(await useCollection(route.params.id as string));

if (!collection.value) {
  throw createError({ statusCode: 404, statusMessage: "Collection not found" });
}

useHead({
  title: collection.value.name || "Collection",
});
</script>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: all 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
  transform: translateX(30px);
}
</style> 
