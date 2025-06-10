<template>
  <div class="h-full w-full relative">
    <!-- Backdrop -->
    <div class="absolute inset-0">
      <img
        :src="bannerUrl"
        class="w-full h-full object-cover opacity-20 blur-xl scale-110"
        :alt="game.mName"
      />
      <div class="absolute inset-0 bg-gradient-to-br from-zinc-900/80 via-zinc-950/90 to-zinc-950"></div>
      <div class="absolute inset-0 bg-[radial-gradient(ellipse_at_70%_30%,_var(--tw-gradient-stops))] from-zinc-800/10 via-transparent to-transparent"></div>
      <div class="absolute inset-0 bg-[radial-gradient(ellipse_at_20%_80%,_var(--tw-gradient-stops))] from-zinc-700/5 via-transparent to-transparent"></div>
    </div>

    <!-- Content -->
    <div class="relative h-full flex flex-col">
      <!-- Header -->
      <div class="p-8 flex items-center">
        <button
          @click="$emit('back')"
          class="group p-3 rounded-lg bg-zinc-800/50 hover:bg-zinc-800 text-zinc-400 hover:text-zinc-200 transition-all duration-300"
        >
          <ArrowLeftIcon class="h-6 w-6 transform group-hover:-translate-x-1 transition-transform duration-300" />
        </button>
      </div>

      <!-- Game Info -->
      <div class="flex-1 p-8 flex gap-12">
        <!-- Left Column -->
        <div class="w-[400px]">
          <div class="relative group">
            <div class="absolute inset-0 bg-blue-500/20 rounded-xl blur-xl transform group-hover:scale-105 transition-transform duration-500"></div>
            <img
              :src="bannerUrl"
              :alt="game.mName"
              class="relative w-[400px] h-[225px] rounded-xl object-cover shadow-2xl transform group-hover:scale-105 transition-transform duration-500"
            />
          </div>
        </div>

        <!-- Right Column -->
        <div class="flex-1 flex flex-col justify-center">
          <h1 class="text-5xl font-display font-bold text-zinc-100 mb-4">
            {{ game.mName }}
          </h1>
          
          <div class="flex items-center gap-4 mb-8">
            <span
              class="px-4 py-2 rounded-full text-sm font-medium"
              :class="[gameStatusTextStyle[games[game.id].status.value.type]]"
            >
              {{ gameStatusText[games[game.id].status.value.type] }}
            </span>
          </div>

          <!-- Action Buttons -->
          <div class="flex items-center gap-4 mb-8">
            <button
              v-if="games[game.id].status.value.type === GameStatusEnum.Installed"
              @click="launchGame"
              class="group px-8 py-4 rounded-xl bg-blue-600 hover:bg-blue-500 text-white font-semibold transition-all duration-300 flex items-center gap-2 text-lg shadow-lg"
            >
              <PlayIcon class="h-5 w-5 transform group-hover:scale-110 transition-transform duration-300" />
              Play Now
            </button>
            <button
              v-else-if="games[game.id].status.value.type === GameStatusEnum.Remote"
              @click="installGame"
              class="group px-8 py-4 rounded-xl bg-green-600 hover:bg-green-500 text-white font-semibold transition-all duration-300 flex items-center gap-2 text-lg shadow-lg"
            >
              <ArrowDownTrayIcon class="h-5 w-5 transform group-hover:scale-110 transition-transform duration-300" />
              Install
            </button>
            <button
              v-else
              disabled
              class="px-8 py-4 rounded-xl bg-zinc-700 text-zinc-400 font-semibold flex items-center gap-2 text-lg shadow-lg"
            >
              <ClockIcon class="h-5 w-5" />
              {{ gameStatusText[games[game.id].status.value.type] }}
            </button>
          </div>

          <div class="prose prose-invert max-w-none mb-8">
            <p class="text-xl text-zinc-300 leading-relaxed">
              {{ game.mDescription || 'No description available.' }}
            </p>
          </div>

          <!-- Additional Info -->
          <div class="grid grid-cols-2 gap-6">
            <div class="bg-zinc-800/50 rounded-xl p-6 backdrop-blur-sm transform hover:scale-105 transition-transform duration-300">
              <h3 class="text-sm font-medium text-zinc-400 mb-2">Status</h3>
              <p class="text-lg text-zinc-100">{{ gameStatusText[games[game.id].status.value.type] }}</p>
            </div>
            <div class="bg-zinc-800/50 rounded-xl p-6 backdrop-blur-sm transform hover:scale-105 transition-transform duration-300">
              <h3 class="text-sm font-medium text-zinc-400 mb-2">Game ID</h3>
              <p class="text-lg text-zinc-100 font-mono">{{ game.id }}</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { 
  ArrowLeftIcon,
  PlayIcon,
  ArrowDownTrayIcon,
  ClockIcon
} from '@heroicons/vue/24/outline';
import { GameStatusEnum, type Game, type GameStatus } from "~/types";
import { invoke } from "@tauri-apps/api/core";

const props = defineProps<{
  game: Game;
  games: { [key: string]: { game: Game; status: Ref<GameStatus, GameStatus> } };
  icons: { [key: string]: string };
}>();

const bannerUrl = await useObject(props.game.mBannerObjectId);

defineEmits<{
  (e: 'back'): void;
}>();

// Style information
const gameStatusTextStyle: { [key in GameStatusEnum]: string } = {
  [GameStatusEnum.Installed]: "bg-green-500/20 text-green-400",
  [GameStatusEnum.Downloading]: "bg-blue-500/20 text-blue-400",
  [GameStatusEnum.Running]: "bg-green-500/20 text-green-400",
  [GameStatusEnum.Remote]: "bg-zinc-500/20 text-zinc-400",
  [GameStatusEnum.Queued]: "bg-blue-500/20 text-blue-400",
  [GameStatusEnum.Updating]: "bg-blue-500/20 text-blue-400",
  [GameStatusEnum.Uninstalling]: "bg-zinc-500/20 text-zinc-400",
  [GameStatusEnum.SetupRequired]: "bg-yellow-500/20 text-yellow-400",
};

const gameStatusText: { [key in GameStatusEnum]: string } = {
  [GameStatusEnum.Remote]: "Not installed",
  [GameStatusEnum.Queued]: "Queued",
  [GameStatusEnum.Downloading]: "Downloading...",
  [GameStatusEnum.Installed]: "Installed",
  [GameStatusEnum.Updating]: "Updating...",
  [GameStatusEnum.Uninstalling]: "Uninstalling...",
  [GameStatusEnum.SetupRequired]: "Setup required",
  [GameStatusEnum.Running]: "Running",
};

const launchGame = async () => {
  await invoke("launch_game", { gameId: props.game.id });
};

const installGame = async () => {
  await invoke("install_game", { gameId: props.game.id });
};
</script> 