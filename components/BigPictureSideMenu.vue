<template>
  <div class="fixed inset-0 z-50 flex pointer-events-none">
    <!-- Overlay -->
    <transition name="fade">
      <div v-if="open" class="absolute inset-0 bg-black/60 pointer-events-auto" @click="$emit('close')"></div>
    </transition>
    <!-- Sidebar -->
    <transition name="slide">
      <nav v-if="open" class="relative z-10 w-72 h-full bg-zinc-900/95 shadow-2xl flex flex-col py-8 px-4 pointer-events-auto">
        <button
          v-for="item in menuItems"
          :key="item.label"
          @click="handleNavigate(item.route)"
          class="flex items-center gap-4 px-4 py-3 rounded-lg text-lg font-semibold text-zinc-100 transition mb-2 menu-item"
          :class="{ 'bg-zinc-800': current === item.route }"
        >
          <component :is="item.icon" class="w-7 h-7" />
          <span>{{ item.label }}</span>
        </button>
      </nav>
    </transition>
  </div>
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

function handleNavigate(route: string) {
  emit('navigate', route);
  setTimeout(() => emit('close'), 100);
}
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
.fade-enter-active, .fade-leave-active {
  transition: opacity 0.2s;
}
.fade-enter-from, .fade-leave-to {
  opacity: 0;
}
.menu-item {
  transition: transform 0.15s cubic-bezier(0.4, 0, 0.2, 1), background 0.15s, color 0.15s;
}
.menu-item:hover {
  transform: scale(1.07) translateX(6px);
  background: #2563eb33; /* subtle blue bg */
  color: #60a5fa;
}
</style> 