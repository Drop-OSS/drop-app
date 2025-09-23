<template>
    <div class="bg-zinc-950 flex flex-col items-center pl-5 px-10 py-8">
      <div class="flex flex-col items-center gap-y-10">
        <Wordmark class="h-8 mb-0.5" />
          <ol class="flex flex-col gap-y-2">
            <NuxtLink
              v-for="(nav, navIdx) in navigation"
              :class="[
                'transition rounded focus:ring-2 ring-blue-600 px-2 uppercase font-display font-semibold text-xl',
                navIdx === currentNavigation
                  ? 'text-zinc-100'
                  : 'text-zinc-400 hover:text-zinc-200',
              ]"
              :href="nav.route"
            >
              {{ nav.label }}
            </NuxtLink>
          </ol>
      </div>
    </div>
</template>

<script setup lang="ts">
import { BellIcon, UserGroupIcon } from "@heroicons/vue/16/solid";
import { AppStatus, type NavigationItem, type QuickActionNav } from "../types";
import { getCurrentWindow } from "@tauri-apps/api/window";

const window = getCurrentWindow();
const state = useAppState();

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
  /*
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
  */
];

const { currentNavigation } = useCurrentNavigationIndex(navigation);

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
