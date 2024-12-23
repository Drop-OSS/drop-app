<template>
  <button
    type="button"
    @click="() => buttonActions[props.status.type]()"
    :class="[
      styles[props.status.type],
      'inline-flex uppercase font-display items-center gap-x-2 rounded-md px-4 py-3 text-md font-semibold shadow-sm focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2',
    ]"
  >
    <component
      :is="buttonIcons[props.status.type]"
      class="-mr-0.5 size-5"
      aria-hidden="true"
    />
    {{ buttonNames[props.status.type] }}
  </button>
</template>

<script setup lang="ts">
import {
  ArrowDownTrayIcon,
  PlayIcon,
  QueueListIcon,
  TrashIcon,
  WrenchIcon,
} from "@heroicons/vue/20/solid";
import type { Component } from "vue";
import { GameStatusEnum, type GameStatus } from "~/types.js";

const props = defineProps<{ status: GameStatus }>();
const emit = defineEmits<{
  (e: "install"): void;
  (e: "play"): void;
  (e: "queue"): void;
}>();

const styles: { [key in GameStatusEnum]: string } = {
  [GameStatusEnum.Remote]:
    "bg-blue-600 text-white hover:bg-blue-500 focus-visible:outline-blue-600",
  [GameStatusEnum.Queued]:
    "bg-zinc-800 text-white hover:bg-zinc-700 focus-visible:outline-zinc-700",
  [GameStatusEnum.Downloading]:
    "bg-zinc-800 text-white hover:bg-zinc-700 focus-visible:outline-zinc-700",
  [GameStatusEnum.SetupRequired]:
    "bg-yellow-600 text-white hover:bg-yellow-500 focus-visible:outline-yellow-600",
  [GameStatusEnum.Installed]:
    "bg-green-600 text-white hover:bg-green-500 focus-visible:outline-green-600",
  [GameStatusEnum.Updating]: "",
  [GameStatusEnum.Uninstalling]: "",
};

const buttonNames: { [key in GameStatusEnum]: string } = {
  [GameStatusEnum.Remote]: "Install",
  [GameStatusEnum.Queued]: "Queued",
  [GameStatusEnum.Downloading]: "Downloading",
  [GameStatusEnum.SetupRequired]: "Setup",
  [GameStatusEnum.Installed]: "Play",
  [GameStatusEnum.Updating]: "Updating",
  [GameStatusEnum.Uninstalling]: "Uninstalling",
};

const buttonIcons: { [key in GameStatusEnum]: Component } = {
  [GameStatusEnum.Remote]: ArrowDownTrayIcon,
  [GameStatusEnum.Queued]: QueueListIcon,
  [GameStatusEnum.Downloading]: ArrowDownTrayIcon,
  [GameStatusEnum.SetupRequired]: WrenchIcon,
  [GameStatusEnum.Installed]: PlayIcon,
  [GameStatusEnum.Updating]: ArrowDownTrayIcon,
  [GameStatusEnum.Uninstalling]: TrashIcon,
};

const buttonActions: { [key in GameStatusEnum]: () => void } = {
  [GameStatusEnum.Remote]: () => emit("install"),
  [GameStatusEnum.Queued]: () => emit("queue"),
  [GameStatusEnum.Downloading]: () => emit("queue"),
  [GameStatusEnum.SetupRequired]: () => {},
  [GameStatusEnum.Installed]: () => emit("play"),
  [GameStatusEnum.Updating]: () => emit("queue"),
  [GameStatusEnum.Uninstalling]: () => {},
};
</script>
