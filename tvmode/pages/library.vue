<template>
  <div class=" p-8">
    <div class="mb-4 flex gap-x-2">
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
          class="block w-full rounded-lg border-0 bg-zinc-800/50 py-2 pl-10 pr-3 text-zinc-100 placeholder:text-zinc-500 focus:bg-zinc-800 focus:ring-2 focus:ring-inset focus:ring-blue-500 sm:text-sm sm:leading-6"
          placeholder="Search library..."
        />
      </div>
      <button
        
        class="p-1 flex items-center justify-center transition-transform duration-300 size-10 hover:scale-110 active:scale-90 rounded-lg bg-zinc-800/50 text-zinc-100"
      >
        <ArrowPathIcon class="size-4" />
      </button>
    </div>
    <div class="grid grid-cols-4 gap-4">
      <NuxtLink
        class="group transition-all duration-300 overflow-hidden bg-zinc-950 p-2 rounded-xl relative focus:scale-105"
        v-for="game in newGames"
        :key="game.id"
        :to="`/library/${game.id}`"
      >
        <div class="h-full z-10 relative bg-zinc-800/40 p-4 rounded-xl">
          <h1 class="text-xl text-zinc-100 font-bold">{{ game.mName }}</h1>
          <p class="text-xs text-zinc-400">{{ game.mShortDescription }}</p>
        </div>
        <img
          class="transition group-focus:blur absolute inset-0 z-0"
          :src="useObject(game.mBannerObjectId)"
        />
      </NuxtLink>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ArrowPathIcon, MagnifyingGlassIcon } from "@heroicons/vue/16/solid";
import { invoke } from "@tauri-apps/api/core";
import type { Game } from "~/types";

const newGames = await invoke<Game[]>("fetch_library", {
  hardRefresh: true,
});
</script>
