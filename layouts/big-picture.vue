<template>
  <div class="h-screen w-screen bg-zinc-950 overflow-hidden">
    <div class="absolute top-4 right-4 z-50">
      <button
        @click="exitBigPicture"
        class="p-2 rounded-lg bg-zinc-800/50 hover:bg-zinc-800 text-zinc-400 hover:text-zinc-200 transition-all duration-300"
      >
        <XMarkIcon class="h-6 w-6" />
      </button>
    </div>
    <slot />
  </div>
</template>

<script setup lang="ts">
import { XMarkIcon } from '@heroicons/vue/24/outline';
import { useRouter } from 'vue-router';

const router = useRouter();

const exitBigPicture = () => {
  router.push('/library');
};
</script> 