<template>
  <div
    @mousedown="() => window.startDragging()"
    class="cursor-pointer bg-gray-950 flex flex-row pl-4 pr-2 py-2"
  >
    <div class="grow inline-flex items-center gap-x-10">
      <Wordmark class="h-8 mb-1" />
      <nav class="inline-flex items-center">
        <ol class="inline-flex items-center gap-x-6">
          <li
            class="transition text-gray-300 hover:text-gray-100 uppercase font-display font-semibold text-md"
            v-for="(nav, navIdx) in navigation"
          >
            {{ nav.label }}
          </li>
        </ol>
      </nav>
    </div>
    <div class="inline-flex items-center">
      <ol class="inline-flex gap-3">
        <li v-for="(item, itemIdx) in quickActions">
          <HeaderWidget
            @click="item.action"
            :notifications="item.notifications"
          >
            <component class="h-5" :is="item.icon" />
          </HeaderWidget>
        </li>
        <HeaderUserWidget />
      </ol>
      <div class="ml-3 inline-flex h-6 gap-x-2">
        <HeaderButton @click="() => window.minimize()">
          <MinusIcon />
        </HeaderButton>
        <HeaderButton @click="() => window.maximize()">
          <svg
            viewBox="0 0 24 24"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
          >
            <rect
              x="6"
              y="6"
              width="12"
              height="12"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
        </HeaderButton>
        <HeaderButton @click="() => window.close()">
          <XMarkIcon />
        </HeaderButton>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  BellIcon,
  MinusIcon,
  UserGroupIcon,
  XMarkIcon,
} from "@heroicons/vue/16/solid";
import type { NavigationItem, QuickActionNav } from "./types";
import HeaderWidget from "./HeaderWidget.vue";
import { getCurrentWindow } from "@tauri-apps/api/window";

const window = getCurrentWindow();

const navigation: Array<NavigationItem> = [
  {
    prefix: "/store",
    route: "/store",
    label: "Store",
  },
  {
    prefix: "/library",
    route: "/library",
    label: "Library",
  },
  {
    prefix: "/community",
    route: "/community",
    label: "Community",
  },
  {
    prefix: "/news",
    route: "/news",
    label: "News",
  },
];

const quickActions: Array<QuickActionNav> = [
  {
    icon: UserGroupIcon,
    action: async () => {},
  },
  {
    icon: BellIcon,
    action: async () => {},
  },
];
</script>
