<template>
  <div class="mx-auto max-w-4xl px-8 py-8">
    <div class="border-b border-zinc-700 py-5">
      <h3 class="text-2xl font-semibold font-display leading-6 text-zinc-100">
        Settings
      </h3>
    </div>
    <div class="mt-8 flex flex-row gap-12">
      <nav class="flex flex-col" aria-label="Sidebar">
        <ul role="list" class="space-y-2">
          <li v-for="(item, itemIdx) in navigation" :key="item.prefix">
            <NuxtLink :href="item.route" :class="[
              itemIdx === currentNavigation
                ? 'bg-zinc-800/70 text-zinc-100 scale-105'
                : 'text-zinc-400 hover:bg-zinc-800/40 hover:text-zinc-200',
              'transition group flex gap-x-3 rounded-xl p-4 text-lg font-semibold',
            ]">
              <component :is="item.icon" :class="[
                itemIdx === currentNavigation
                  ? 'text-blue-400'
                  : 'text-zinc-400 group-hover:text-blue-300',
                'transition h-8 w-8 shrink-0',
              ]" aria-hidden="true" />
              {{ item.label }}
            </NuxtLink>
          </li>
        </ul>
      </nav>
      <div class="grow bg-zinc-900/70 rounded-xl p-8 min-h-[400px]">
        <NuxtPage />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  ArrowDownTrayIcon,
  CubeIcon,
  HomeIcon,
  RectangleGroupIcon,
  BugAntIcon,
} from "@heroicons/vue/16/solid";
import type { Component } from "vue";
import type { NavigationItem } from "~/types";
import { platform } from '@tauri-apps/plugin-os';
import { invoke } from "@tauri-apps/api/core";
import { UserIcon } from "@heroicons/vue/20/solid";

const systemData = await invoke<{
  clientId: string;
  baseUrl: string;
  dataDir: string;
  logLevel: string;
}>("fetch_system_data");

const isDebugMode = ref(systemData.logLevel.toLowerCase() === "debug");
const debugRevealed = ref(false);

onMounted(() => {
  window.addEventListener('keydown', (e) => {
    if (e.key === 'Shift') {
      isDebugMode.value = true;
      debugRevealed.value = true;
    }
  });
  
  window.addEventListener('keyup', (e) => {
    if (e.key === 'Shift') {
      isDebugMode.value = debugRevealed.value || systemData.logLevel.toLowerCase() === "debug";
    }
  });

  // Reset debug reveal when leaving the settings page
  const router = useRouter();
  router.beforeEach((to) => {
    if (!to.path.startsWith('/big-picture/settings')) {
      debugRevealed.value = false;
      isDebugMode.value = systemData.logLevel.toLowerCase() === "debug";
    }
  });
});

const navigation = computed(() => [
  {
    label: "Home",
    route: "/big-picture/settings",
    prefix: "/big-picture/settings",
    icon: HomeIcon,
  },
  {
    label: "Interface", 
    route: "/big-picture/settings/interface",
    prefix: "/big-picture/settings/interface",
    icon: RectangleGroupIcon,
  },
  {
    label: "Downloads",
    route: "/big-picture/settings/downloads",
    prefix: "/big-picture/settings/downloads",
    icon: ArrowDownTrayIcon,
  },
  {
    label: "Account",
    route: "/big-picture/settings/account",
    prefix: "/big-picture/settings/account",
    icon: UserIcon
  },
  ...(isDebugMode.value ? [{
    label: "Debug Info",
    route: "/big-picture/settings/debug", 
    prefix: "/big-picture/settings/debug",
    icon: BugAntIcon,
  }] : []),
]);

const currentPlatform = platform();

const {currentNavigation} = useCurrentNavigationIndex(navigation.value);

watch(navigation, (newNav) => {
  currentNavigation.value = useCurrentNavigationIndex(newNav).currentNavigation.value;
});
</script>
