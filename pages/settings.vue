<template>
  <div class="mx-auto max-w-7xl px-8">
    <div class="border-b border-zinc-700 py-5">
      <h3 class="text-base font-semibold font-display leading-6 text-zinc-100">
        Settings
      </h3>
    </div>
    <div class="mt-5 flex flex-row gap-12">
      <nav class="flex flex-col" aria-label="Sidebar">
        <ul role="list" class="-mx-2 space-y-1">
          <li v-for="(item, itemIdx) in navigation" :key="item.prefix">
            <NuxtLink
              :href="item.route"
              :class="[
                itemIdx === currentPageIndex
                  ? 'bg-zinc-800/50 text-blue-600'
                  : 'text-zinc-400 hover:bg-zinc-800/30 hover:text-blue-600',
                'transition group flex gap-x-3 rounded-md p-2 pr-12 text-sm font-semibold leading-6',
              ]"
            >
              <component
                :is="item.icon"
                :class="[
                  itemIdx === currentPageIndex
                    ? 'text-blue-600'
                    : 'text-zinc-400 group-hover:text-blue-600',
                  'transition h-6 w-6 shrink-0',
                ]"
                aria-hidden="true"
              />
              {{ item.label }}
            </NuxtLink>
          </li>
        </ul>
      </nav>
      <div class="grow">
        <NuxtPage />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  ArrowDownTrayIcon,
  HomeIcon,
  RectangleGroupIcon,
} from "@heroicons/vue/16/solid";
import type { Component } from "vue";
import type { NavigationItem } from "~/types";

const navigation: Array<NavigationItem & { icon: Component }> = [
  {
    label: "Home",
    route: "/settings",
    prefix: "/settings",
    icon: HomeIcon,
  },
  {
    label: "Interface",
    route: "/settings/interface",
    prefix: "/settings/interface",
    icon: RectangleGroupIcon,
  },
  {
    label: "Downloads",
    route: "/settings/downloads",
    prefix: "/settings/downloads",
    icon: ArrowDownTrayIcon,
  },
];

const currentPageIndex = useCurrentNavigationIndex(navigation);
</script>
