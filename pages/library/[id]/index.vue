<template>
  <div
    class="mx-auto w-full relative flex flex-col justify-center pt-64 z-10 overflow-hidden"
  >
    <!-- banner image -->
    <div class="absolute flex top-0 h-fit inset-x-0 z-[-20]">
      <img :src="bannerUrl" class="w-full h-auto object-cover" />
      <h1
        class="absolute inset-x-0 w-full text-center top-32 -translate-y-[50%] text-4xl text-zinc-100 font-bold font-display z-50"
      >
        {{ game.mName }}
      </h1>
      <div
        class="absolute inset-0 bg-gradient-to-b from-transparent to-50% to-zinc-900"
      />
    </div>
    <!-- main page -->
    <div class="w-full min-h-screen mx-auto bg-zinc-900 px-5 py-6">
      <!-- game toolbar -->
      <div>
        <GameButton v-model="status" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Game } from "@prisma/client";
import { invoke } from "@tauri-apps/api/core";

const route = useRoute();
const id = route.params.id;

const raw: { game: Game; status: any } = JSON.parse(
  await invoke<string>("fetch_game", { id: id })
);
const game = ref(raw.game);
const status = ref(raw.status);

const bannerUrl = await useObject(game.value.mBannerId);
</script>
