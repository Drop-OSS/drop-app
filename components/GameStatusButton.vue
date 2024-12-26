<template>
  <div class="inline-flex">
    <button type="button" @click="() => buttonActions[props.status.type]()" :class="[
      styles[props.status.type],
      showDropdown ? 'rounded-l-md' : 'rounded-md',
      'inline-flex uppercase font-display items-center gap-x-2 px-4 py-3 text-md font-semibold shadow-sm focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2',
    ]">
      <component :is="buttonIcons[props.status.type]" class="-mr-0.5 size-5" aria-hidden="true" />
      {{ buttonNames[props.status.type] }}
    </button>
    <Menu v-if="showDropdown" as="div" class="relative inline-block text-left grow">
      <div class="h-full">
        <MenuButton
          class="inline-flex w-full h-full justify-center items-center rounded-r-md bg-zinc-800 px-1 py-2 text-sm font-semibold text-zinc-100 shadow-sm ring-1 ring-inset ring-zinc-600 hover:bg-zinc-800">

          <ChevronDownIcon class="size-5 text-gray-400" aria-hidden="true" />
        </MenuButton>
      </div>

      <transition enter-active-class="transition ease-out duration-100" enter-from-class="transform opacity-0 scale-95"
        enter-to-class="transform opacity-100 scale-100" leave-active-class="transition ease-in duration-75"
        leave-from-class="transform opacity-100 scale-100" leave-to-class="transform opacity-0 scale-95">
        <MenuItems
          class="absolute right-0 z-50 mt-2 w-32 origin-top-right rounded-md bg-zinc-900 shadow-lg ring-1 ring-zinc-100/5 focus:outline-none">
          <div class="py-1">
            <MenuItem v-slot="{ active }">
            <button @click="() => emit('uninstall')"
              :class="[active ? 'bg-zinc-800 text-zinc-100 outline-none' : 'text-zinc-400', 'w-full block px-4 py-2 text-sm inline-flex justify-between']">Uninstall
              <TrashIcon class="size-5" />
            </button>
            </MenuItem>
          </div>
        </MenuItems>
      </transition>
    </Menu>
  </div>
</template>

<script setup lang="ts">
import {
  ArrowDownTrayIcon,
  ChevronDownIcon,
  PlayIcon,
  QueueListIcon,
  TrashIcon,
  WrenchIcon,
} from "@heroicons/vue/20/solid";

import type { Component } from "vue";
import { GameStatusEnum, type GameStatus } from "~/types.js";
import { Menu, MenuButton, MenuItem, MenuItems } from '@headlessui/vue'

const props = defineProps<{ status: GameStatus }>();
const emit = defineEmits<{
  (e: "install"): void;
  (e: "play"): void;
  (e: "queue"): void;
  (e: "uninstall"): void;
}>();

const showDropdown = computed(() => props.status.type === GameStatusEnum.Installed);

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
  [GameStatusEnum.SetupRequired]: () => { },
  [GameStatusEnum.Installed]: () => emit("play"),
  [GameStatusEnum.Updating]: () => emit("queue"),
  [GameStatusEnum.Uninstalling]: () => { },
};
</script>
