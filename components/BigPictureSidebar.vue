<template>
  <div class="bg-zinc-900 border-r border-zinc-800 p-6 space-y-4 transition-all duration-500 ease-out">
    <!-- Logo Section -->
    <div class="mb-8 transform transition-all duration-300 hover:scale-105">
      <Wordmark class="h-10" />
    </div>
    
    <!-- Navigation Tiles -->
    <div class="space-y-3">
      <BigPictureNavTile
        v-for="item in navigationItems"
        :key="item.route"
        :item="item"
        :is-active="currentPage === item.route"
        @click="navigateTo(item.route)"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  BookOpenIcon,
  BuildingStorefrontIcon,
  QueueListIcon,
  Cog6ToothIcon,
} from "@heroicons/vue/24/outline";
import { useBigPictureMode, setBigPicturePage } from "~/composables/big-picture";
import { useQueueState } from "~/composables/downloads";
import BigPictureNavTile from "~/components/BigPictureNavTile.vue";

const router = useRouter();
const state = useBigPictureMode();
const currentPage = computed(() => state.value.currentPage);
const queue = useQueueState();

const navigationItems = computed(() => [
  {
    icon: BookOpenIcon,
    label: "Library",
    route: "/big-picture/library",
    description: "Your game collection",
    notifications: undefined
  },
  {
    icon: BuildingStorefrontIcon,
    label: "Store",
    route: "/big-picture/store",
    description: "Discover new games",
    notifications: undefined
  },
  {
    icon: QueueListIcon,
    label: "Downloads",
    route: "/big-picture/queue",
    description: "Manage downloads",
    notifications: queue.value.queue.length > 0 ? queue.value.queue.length : undefined
  },
  {
    icon: Cog6ToothIcon,
    label: "Settings",
    route: "/big-picture/settings",
    description: "App configuration",
    notifications: undefined
  }
]);

const navigateTo = async (route: string) => {
  setBigPicturePage(route);
  await router.push(route);
};
</script> 