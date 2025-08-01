<template>
  <div class="px-6 py-6 flex items-center justify-between transition-all duration-300">
    <!-- Right Section -->
    <div class="flex items-center justify-end w-full space-x-6">
      <!-- User Dropdown -->
      <Menu v-if="state.user" as="div" class="relative inline-block">
        <MenuButton class="flex items-center space-x-4 bg-zinc-900/80 backdrop-blur-xl hover:bg-zinc-800/80 rounded-xl p-4 transition-colors duration-200 border-2 border-zinc-700/50 hover:border-zinc-600">
          <img 
            :src="profilePictureUrl" 
            class="w-14 h-14 rounded-xl border-2 border-zinc-600" 
            alt="Profile"
          />
          <div class="text-right">
            <p class="text-lg font-semibold text-zinc-100">
              {{ state.user.displayName }}
            </p>
            <p class="text-sm text-zinc-400">
              {{ state.user.username }}
            </p>
          </div>
          <ChevronDownIcon class="h-6 w-6 text-zinc-400 ml-3" />
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
            class="absolute bg-zinc-900 right-0 top-16 z-50 w-72 origin-top-right focus:outline-none shadow-lg rounded-xl border-2 border-zinc-800"
          >
            <div class="flex-col gap-y-3 p-2">
              <NuxtLink
                to="/id/me"
                class="transition inline-flex items-center w-full py-4 px-6 hover:bg-zinc-800 rounded-lg"
              >
                <div class="inline-flex items-center text-zinc-300">
                  <img :src="profilePictureUrl" class="w-8 h-8 rounded-lg" />
                  <span class="ml-3 text-base font-bold">{{
                    state.user.displayName
                  }}</span>
                </div>
              </NuxtLink>
              <div class="h-0.5 rounded-full w-full bg-zinc-800 mx-2" />
              <div class="flex flex-col mb-1">
                <MenuItem v-slot="{ active }">
                  <button
                    @click="exitBigPictureMode"
                    :class="[
                      active ? 'bg-zinc-800 text-zinc-100' : 'text-zinc-400',
                      'transition text-left block px-6 py-4 text-base rounded-lg',
                    ]"
                  >
                    <div class="flex items-center space-x-3">
                      <XMarkIcon class="h-5 w-5" />
                      <span>Exit Big Picture Mode</span>
                    </div>
                  </button>
                </MenuItem>
              </div>
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