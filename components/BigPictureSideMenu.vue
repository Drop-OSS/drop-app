<template>
  <transition name="slide">
    <div v-if="open" class="fixed inset-0 z-50 flex">
      <!-- Overlay -->
      <div class="absolute inset-0 bg-black/60" @click="$emit('close')"></div>
      <!-- Sidebar -->
      <nav class="relative z-10 w-72 h-full bg-zinc-900/95 shadow-2xl flex flex-col py-8 px-4">
        <button
          v-for="item in menuItems"
          :key="item.label"
          @click="$emit('navigate', item.route)"
          class="flex items-center gap-4 px-4 py-3 rounded-lg text-lg font-semibold text-zinc-100 hover:bg-zinc-800 transition mb-2"
          :class="{ 'bg-zinc-800': current === item.route }"
        >
          <component :is="item.icon" class="w-7 h-7" />
          <span>{{ item.label }}</span>
        </button>
      </nav>
    </div>
  </transition>
</template>

<script setup lang="ts">
import { Squares2X2Icon, ShoppingBagIcon, UsersIcon, NewspaperIcon, Cog6ToothIcon } from '@heroicons/vue/24/outline';

const props = defineProps<{ open: boolean; current: string }>();
const emit = defineEmits(['close', 'navigate']);

const menuItems = [
  { label: 'Library', route: '/big-picture', icon: Squares2X2Icon },
  { label: 'Store', route: '/big-picture/store', icon: ShoppingBagIcon },
  { label: 'Community', route: '/big-picture/community', icon: UsersIcon },
  { label: 'News', route: '/big-picture/news', icon: NewspaperIcon },
  { label: 'Settings', route: '/big-picture/settings', icon: Cog6ToothIcon },
];
</script>

<style scoped>
.slide-enter-active, .slide-leave-active {
  transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}
.slide-enter-from {
  transform: translateX(-100%);
}
.slide-leave-to {
  transform: translateX(-100%);
}
</style> 