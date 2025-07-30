<template>
  <div class="bg-zinc-900 border-b border-zinc-800 px-6 py-4 flex items-center justify-between transition-all duration-300">
    <!-- Left Section -->
    <div class="flex items-center space-x-6">
      <!-- Current Page Title -->
      <div class="transform transition-all duration-300">
        <h1 class="text-2xl font-bold font-display text-zinc-100">
          {{ currentPageTitle }}
        </h1>
        <p class="text-sm text-zinc-400 mt-1">
          {{ currentPageDescription }}
        </p>
      </div>
    </div>
    
    <!-- Right Section -->
    <div class="flex items-center space-x-4">
      <!-- User Dropdown -->
      <Menu v-if="state.user" as="div" class="relative inline-block">
        <MenuButton class="flex items-center space-x-3 hover:bg-zinc-800 rounded-lg p-2 transition-colors duration-200">
          <img 
            :src="profilePictureUrl" 
            class="w-10 h-10 rounded-lg border-2 border-zinc-700" 
            alt="Profile"
          />
          <div class="text-right">
            <p class="text-sm font-semibold text-zinc-100">
              {{ state.user.displayName }}
            </p>
            <p class="text-xs text-zinc-400">
              {{ state.user.username }}
            </p>
          </div>
          <ChevronDownIcon class="h-5 w-5 text-zinc-400 ml-2" />
        </MenuButton>

        <transition
          enter-active-class="transition ease-out duration-100"
          enter-from-class="transform opacity-0 scale-95"
          enter-to-class="transform opacity-100 scale-100"
          leave-active-class="transition ease-in duration-75"
          leave-from-class="transform opacity-100 scale-100"
          leave-to-class="transform opacity-0 scale-95"
        >
          <MenuItems
            class="absolute bg-zinc-900 right-0 top-12 z-50 w-56 origin-top-right focus:outline-none shadow-md rounded-lg border border-zinc-800"
          >
            <div class="flex-col gap-y-2 p-2">
              <MenuItem v-slot="{ active }">
                <button
                  @click="exitBigPictureMode"
                  :class="[
                    active ? 'bg-zinc-800 text-zinc-100' : 'text-zinc-400',
                    'transition text-left block w-full px-4 py-3 text-sm font-semibold text-red-400 hover:text-red-300',
                  ]"
                >
                  <div class="flex items-center space-x-2">
                    <XMarkIcon class="h-5 w-5" />
                    <span>Exit Big Picture Mode</span>
                  </div>
                </button>
              </MenuItem>
            </div>
          </MenuItems>
        </transition>
      </Menu>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Menu, MenuButton, MenuItem, MenuItems } from "@headlessui/vue";
import { XMarkIcon } from "@heroicons/vue/24/outline";
import { ChevronDownIcon } from "@heroicons/vue/16/solid";
import { useBigPictureMode, exitBigPictureMode } from "~/composables/big-picture";
import { useAppState } from "~/composables/app-state";

const bigPictureState = useBigPictureMode();
const currentPage = computed(() => bigPictureState.value.currentPage);
const state = useAppState();

const profilePictureUrl: string = await useObject(
  state.value.user?.profilePictureObjectId ?? ""
);

const currentPageTitle = computed(() => {
  switch (currentPage.value) {
    case "/big-picture/library":
      return "Library";
    case "/big-picture/store":
      return "Store";
    case "/big-picture/queue":
      return "Downloads";
    case "/big-picture/settings":
      return "Settings";
    default:
      return "Drop";
  }
});

const currentPageDescription = computed(() => {
  switch (currentPage.value) {
    case "/big-picture/library":
      return "Your game collection";
    case "/big-picture/store":
      return "Discover new games";
    case "/big-picture/queue":
      return "Manage downloads";
    case "/big-picture/settings":
      return "App configuration";
    default:
      return "Game distribution platform";
  }
});
</script> 