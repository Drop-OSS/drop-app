<template>
  <div class="inline-flex divide-x divide-zinc-900">
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
        <MenuButton :class="[
          styles[props.status.type],
          'inline-flex w-full h-full justify-center items-center rounded-r-md px-1 py-2 text-sm font-semibold shadow-sm'
        ]">
          <ChevronDownIcon class="size-5" aria-hidden="true" />
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

    <button v-if="showInfoButton" 
      @click="infoOpen = true"
      class="px-2 bg-yellow-600 hover:bg-yellow-500 text-white rounded-md ml-2 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-yellow-600">
      <InformationCircleIcon class="size-5" />
    </button>

    <TransitionRoot as="template" :show="infoOpen">
      <Dialog as="div" class="relative z-50" @close="infoOpen = false">
        <TransitionChild enter="ease-out duration-300" enter-from="opacity-0" enter-to="opacity-100"
          leave="ease-in duration-200" leave-from="opacity-100" leave-to="opacity-0">
          <div class="fixed inset-0 bg-zinc-950/75 transition-opacity" />
        </TransitionChild>

        <div class="fixed inset-0 z-10 w-screen overflow-y-auto">
          <div class="flex min-h-full items-end justify-center p-4 text-center sm:items-center sm:p-0">
            <TransitionChild enter="ease-out duration-300"
              enter-from="opacity-0 translate-y-4 sm:translate-y-0 sm:scale-95"
              enter-to="opacity-100 translate-y-0 sm:scale-100" leave="ease-in duration-200"
              leave-from="opacity-100 translate-y-0 sm:scale-100"
              leave-to="opacity-0 translate-y-4 sm:translate-y-0 sm:scale-95">
              <DialogPanel
                class="relative transform overflow-hidden rounded-lg bg-zinc-900 px-4 pb-4 pt-5 text-left shadow-xl transition-all sm:my-8 sm:w-full sm:max-w-lg sm:p-6">
                <div class="space-y-4">
                  <div class="text-center">
                    <DialogTitle as="h3" class="text-xl font-semibold leading-6 text-zinc-100">
                      Game Information
                    </DialogTitle>
                  </div>
                  
                  <div class="space-y-3 text-left">
                    <div class="flex justify-between items-center">
                      <span class="text-zinc-400">Game ID:</span>
                      <span class="text-zinc-100">{{ displayGameId }}</span>
                    </div>
                    
                    <div v-if="setupRequired" class="flex justify-between items-center">
                      <span class="text-zinc-400">Status:</span>
                      <span class="text-yellow-500">Setup Required</span>
                    </div>
                  </div>
                  
                  <div class="mt-5">
                    <button type="button"
                      class="w-full rounded-md bg-zinc-800 px-3 py-2 text-sm font-semibold text-zinc-100 hover:bg-zinc-700"
                      @click="infoOpen = false">Close</button>
                  </div>
                </div>
              </DialogPanel>
            </TransitionChild>
          </div>
        </div>
      </Dialog>
    </TransitionRoot>
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
  InformationCircleIcon,
} from "@heroicons/vue/20/solid";
import { Dialog, DialogPanel, DialogTitle, TransitionChild, TransitionRoot } from '@headlessui/vue'

import type { Component } from "vue";
import { GameStatusEnum, type GameStatus } from "~/types.js";
import { Menu, MenuButton, MenuItem, MenuItems } from '@headlessui/vue'
import { useRoute } from 'vue-router';
import { invoke } from "@tauri-apps/api/core";

const props = defineProps<{ 
  status: GameStatus;
  gameId?: string;
}>();

const route = useRoute();
const displayGameId = computed(() => props.gameId || route.params.id as string);

const infoOpen = ref(false);
const showInfoButton = computed(() => 
  props.status.type === GameStatusEnum.Installed || 
  props.status.type === GameStatusEnum.SetupRequired
);

const setupRequired = computed(() => props.status.type === GameStatusEnum.SetupRequired);

const emit = defineEmits<{
  (e: "install"): void;
  (e: "launch"): void;
  (e: "queue"): void;
  (e: "uninstall"): void;
  (e: "kill"): void;
}>();

const showDropdown = computed(() => props.status.type === GameStatusEnum.Installed || props.status.type === GameStatusEnum.SetupRequired);

const styles: { [key in GameStatusEnum]: string } = {
  [GameStatusEnum.Remote]: "bg-blue-600 text-white hover:bg-blue-500 focus-visible:outline-blue-600",
  [GameStatusEnum.Queued]: "bg-zinc-800 text-white hover:bg-zinc-700 focus-visible:outline-zinc-700",
  [GameStatusEnum.Downloading]: "bg-zinc-800 text-white hover:bg-zinc-700 focus-visible:outline-zinc-700",
  [GameStatusEnum.SetupRequired]: "bg-yellow-600 text-white hover:bg-yellow-500 focus-visible:outline-yellow-600",
  [GameStatusEnum.Installed]: "bg-green-600 text-white hover:bg-green-500 focus-visible:outline-green-600",
  [GameStatusEnum.Updating]: "bg-zinc-800 text-white hover:bg-zinc-700 focus-visible:outline-zinc-700",
  [GameStatusEnum.Uninstalling]: "bg-zinc-800 text-white hover:bg-zinc-700 focus-visible:outline-zinc-700",
  [GameStatusEnum.Running]: "bg-zinc-800 text-white focus-visible:outline-zinc-700"
};

const buttonNames: { [key in GameStatusEnum]: string } = {
  [GameStatusEnum.Remote]: "Install",
  [GameStatusEnum.Queued]: "Queued",
  [GameStatusEnum.Downloading]: "Downloading",
  [GameStatusEnum.SetupRequired]: "Setup",
  [GameStatusEnum.Installed]: "Play",
  [GameStatusEnum.Updating]: "Updating",
  [GameStatusEnum.Uninstalling]: "Uninstalling",
  [GameStatusEnum.Running]: "Stop"
};

const buttonIcons: { [key in GameStatusEnum]: Component } = {
  [GameStatusEnum.Remote]: ArrowDownTrayIcon,
  [GameStatusEnum.Queued]: QueueListIcon,
  [GameStatusEnum.Downloading]: ArrowDownTrayIcon,
  [GameStatusEnum.SetupRequired]: WrenchIcon,
  [GameStatusEnum.Installed]: PlayIcon,
  [GameStatusEnum.Updating]: ArrowDownTrayIcon,
  [GameStatusEnum.Uninstalling]: TrashIcon,
  [GameStatusEnum.Running]: PlayIcon
};

const buttonActions: { [key in GameStatusEnum]: () => void } = {
  [GameStatusEnum.Remote]: () => emit("install"),
  [GameStatusEnum.Queued]: () => emit("queue"),
  [GameStatusEnum.Downloading]: () => emit("queue"),
  [GameStatusEnum.SetupRequired]: () => emit("launch"),
  [GameStatusEnum.Installed]: () => emit("launch"),
  [GameStatusEnum.Updating]: () => emit("queue"),
  [GameStatusEnum.Uninstalling]: () => { },
  [GameStatusEnum.Running]: () => emit("kill")
};
</script>
