<template>
  <button
    type="button"
    @click="() => buttonActions[props.status]()"
    :class="[
      styles[props.status],
      'inline-flex uppercase font-display items-center gap-x-2 rounded-md px-4 py-3 text-md font-semibold shadow-sm focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2',
    ]"
  >
    <component
      :is="buttonIcons[props.status]"
      class="-mr-0.5 size-5"
      aria-hidden="true"
    />
    {{ buttonNames[props.status] }}
  </button>
</template>

<script setup lang="ts">
import {
  ArrowDownTrayIcon,
  PlayIcon,
  QueueListIcon,
  TrashIcon,
} from "@heroicons/vue/20/solid";
import type { Component } from "vue";
import { GameStatus } from "~/types.js";

const props = defineProps<{ status: GameStatus }>();
const emit = defineEmits<{
  (e: "install"): void;
  (e: "cancel"): void;
  (e: "play"): void;
}>();

const styles: { [key in GameStatus]: string } = {
  [GameStatus.Remote]:
    "bg-blue-600 text-white hover:bg-blue-500 focus-visible:outline-blue-600",
  [GameStatus.Queued]:
    "bg-zinc-800 text-white hover:bg-zinc-700 focus-visible:outline-zinc-700",
  [GameStatus.Downloading]:
    "bg-zinc-800 text-white hover:bg-zinc-700 focus-visible:outline-zinc-700",
  [GameStatus.Installed]:
    "bg-green-600 text-white hover:bg-green-500 focus-visible:outline-green-600",
  [GameStatus.Updating]: "",
  [GameStatus.Uninstalling]: "",
};

const buttonNames: { [key in GameStatus]: string } = {
  [GameStatus.Remote]: "Install",
  [GameStatus.Queued]: "Queued",
  [GameStatus.Downloading]: "Downloading",
  [GameStatus.Installed]: "Play",
  [GameStatus.Updating]: "Updating",
  [GameStatus.Uninstalling]: "Uninstalling",
};

const buttonIcons: { [key in GameStatus]: Component } = {
  [GameStatus.Remote]: ArrowDownTrayIcon,
  [GameStatus.Queued]: QueueListIcon,
  [GameStatus.Downloading]: ArrowDownTrayIcon,
  [GameStatus.Installed]: PlayIcon,
  [GameStatus.Updating]: ArrowDownTrayIcon,
  [GameStatus.Uninstalling]: TrashIcon,
};

const buttonActions: { [key in GameStatus]: () => void } = {
  [GameStatus.Remote]: () => emit("install"),
  [GameStatus.Queued]: () => emit("cancel"),
  [GameStatus.Downloading]: () => emit("cancel"),
  [GameStatus.Installed]: () => emit("play"),
  [GameStatus.Updating]: () => emit("cancel"),
  [GameStatus.Uninstalling]: () => {},
};
</script>
