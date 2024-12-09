<template>
  <div class="h-16 bg-zinc-950 flex flex-row justify-between">
    <div class="flex flex-row grow items-center pl-5 pr-2 py-3">
      <div class="inline-flex items-center gap-x-10">
        <NuxtLink to="/store">
          <Wordmark class="h-8 mb-0.5" />
        </NuxtLink>
        <nav class="inline-flex items-center mt-0.5">
          <ol class="inline-flex items-center gap-x-6">
            <NuxtLink
              v-for="(nav, navIdx) in navigation"
              :class="[
                'transition  uppercase font-display font-semibold text-md',
                navIdx === currentPageIndex
                  ? 'text-zinc-100'
                  : 'text-zinc-400 hover:text-zinc-200',
              ]"
              :href="nav.route"
            >
              {{ nav.label }}
            </NuxtLink>
          </ol>
        </nav>
      </div>
      <div
        @mousedown="() => window.startDragging()"
        class="flex cursor-pointer grow h-full"
      />
      <div class="inline-flex items-center">
        <ol class="inline-flex gap-3">
          <HeaderQueueWidget
            :object="currentQueueObject"
          />
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
import type { NavigationItem, QuickActionNav } from "../types";
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

const currentPageIndex = useCurrentNavigationIndex(navigation);

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

const queue = useQueueState();
const currentQueueObject = computed(() => queue.value.queue.at(0));
</script>
