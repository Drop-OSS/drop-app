<template>
  <div class="flex h-screen w-screen bg-zinc-950 overflow-hidden">
    <!-- Intro Animation -->
    <BigPictureIntro :show="bigPictureState.showIntro" />
    
    <!-- Sidebar -->
    <BigPictureSidebar class="w-80 flex-shrink-0" />
    
    <!-- Main Content Area -->
    <div class="flex flex-col flex-1 min-w-0">
      <!-- Header -->
      <BigPictureHeader class="h-20 flex-shrink-0" />
      
      <!-- Content -->
      <div class="flex-1 overflow-hidden">
        <div class="h-full overflow-y-auto">
          <Transition
            mode="out-in"
            enter-active-class="transition-all duration-300 ease-out"
            enter-from-class="opacity-0 translate-x-4"
            enter-to-class="opacity-100 translate-x-0"
            leave-active-class="transition-all duration-200 ease-in"
            leave-from-class="opacity-100 translate-x-0"
            leave-to-class="opacity-0 translate-x-4"
          >
            <slot />
          </Transition>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useBigPictureMode } from "~/composables/big-picture";

const bigPictureState = useBigPictureMode();
</script>
