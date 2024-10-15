<template>
  <div
    class="mx-auto w-full relative flex flex-col justify-center pt-64 z-10 overflow-hidden"
  >
    <!-- banner image -->
    <div class="absolute flex top-0 h-fit inset-x-0 -z-[20]">
      <img :src="bannerUrl" class="w-full h-auto object-cover" />
      <div
        class="absolute inset-0 bg-gradient-to-b from-transparent to-50% to-zinc-900"
      />
    </div>
    <!-- main page -->
    <div class="w-full min-h-screen mx-auto bg-zinc-900 px-16 py-12"></div>
  </div>
</template>

<script setup lang="ts">
import type { Game } from "@prisma/client";
import { invoke } from "@tauri-apps/api/core";

const route = useRoute();
const id = route.params.id;

const rawGame = await invoke<string>("fetch_game", { id: id });
const game: Game = JSON.parse(rawGame);

const bannerUrl = await useObject(game.mBannerId);
</script>
