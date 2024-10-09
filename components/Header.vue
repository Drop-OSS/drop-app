<template>
  <div class="h-16 cursor-pointer bg-gray-950 flex flex-row justify-between">
    <div
      @mousedown="() => window.startDragging()"
      class="flex flex-row grow items-center justify-between pl-5 pr-2 py-3"
    >
      <div class="inline-flex items-center gap-x-10">
        <Wordmark class="h-8 mb-0.5" />
        <nav class="inline-flex items-center mt-0.5">
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
      </div>
    </div>
    <WindowControl class="h-16 w-16 p-4" />
  </div>
</template>

<script setup lang="ts">
import { BellIcon, UserGroupIcon } from "@heroicons/vue/16/solid";
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
