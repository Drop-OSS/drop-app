<template>
  <button
    @click="$emit('click')"
    :class="[
      'w-full p-4 rounded-xl transition-all duration-200 transform hover:scale-105 focus:outline-none focus:ring-2 focus:ring-blue-600 focus:ring-offset-2 focus:ring-offset-zinc-900',
      isActive
        ? 'bg-blue-600 text-white shadow-lg shadow-blue-600/25'
        : 'bg-zinc-800 text-zinc-300 hover:bg-zinc-700 hover:text-zinc-100'
    ]"
  >
    <div class="flex items-center space-x-4">
      <!-- Icon -->
      <div class="flex-shrink-0">
        <component 
          :is="item.icon" 
          :class="[
            'h-8 w-8 transition-colors duration-200',
            isActive ? 'text-white' : 'text-zinc-400 group-hover:text-zinc-200'
          ]" 
        />
      </div>
      
      <!-- Content -->
      <div class="flex-1 min-w-0 text-left">
        <div class="flex items-center justify-between">
          <h3 class="text-lg font-semibold font-display truncate">
            {{ item.label }}
          </h3>
          <!-- Notification Badge -->
          <div 
            v-if="item.notifications && item.notifications > 0"
            class="flex-shrink-0 ml-2 bg-red-500 text-white text-xs font-bold rounded-full h-6 w-6 flex items-center justify-center"
          >
            {{ item.notifications > 99 ? '99+' : item.notifications }}
          </div>
        </div>
        <p 
          :class="[
            'text-sm mt-1 truncate',
            isActive ? 'text-blue-100' : 'text-zinc-500 group-hover:text-zinc-400'
          ]"
        >
          {{ item.description }}
        </p>
      </div>
    </div>
  </button>
</template>

<script setup lang="ts">
import type { BigPictureNavItem } from "~/composables/big-picture";

defineProps<{
  item: BigPictureNavItem;
  isActive: boolean;
}>();

defineEmits<{
  click: [];
}>();
</script> 