<template>
  <div class="p-8">
    <!-- Page Header -->
    <div class="mb-8">
      <h2 class="text-3xl font-bold font-display text-zinc-100 mb-2">
        Settings
      </h2>
      <p class="text-lg text-zinc-400">
        App configuration
      </p>
    </div>

    <!-- Coming Soon -->
    <div class="flex flex-col items-center justify-center py-16">
      <div class="text-center">
        <Cog6ToothIcon class="h-24 w-24 text-zinc-600 mx-auto mb-6" />
        <h3 class="text-2xl font-semibold text-zinc-300 mb-2">
          Settings Coming Soon
        </h3>
        <p class="text-zinc-500 mb-6">
          Settings will be available in a future update
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
import { Cog6ToothIcon, BookOpenIcon } from "@heroicons/vue/24/outline";

definePageMeta({
  layout: "big-picture"
});
</script> 