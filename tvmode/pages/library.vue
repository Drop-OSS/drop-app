<template>
  <div class="grid grid-cols-4 gap-4 p-8">
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
</template>

<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import type { Game } from "~/types";

const newGames = await invoke<Game[]>("fetch_library", {
  hardRefresh: true,
});
</script>
