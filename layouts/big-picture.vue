<template>
  <div class="flex h-screen w-screen bg-zinc-950 overflow-hidden">
    <!-- Intro Animation -->
    <BigPictureIntro :show="bigPictureState.showIntro" />
    
    <!-- Sidebar -->
    <BigPictureSidebar class="w-80 flex-shrink-0" />
    
    <!-- Main Content Area -->
    <div class="flex flex-col flex-1 min-w-0 relative">
      <!-- Content -->
      <div class="flex-1 overflow-hidden">
        <div class="h-full overflow-y-auto scrollbar-hide">
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
      
      <!-- Header (positioned on top) -->
      <BigPictureHeader />
    </div>

    <!-- Global Install Modal for Big Picture Mode -->
    <BigPictureInstallModal ref="installModalRef" />
  </div>
</template>

<script setup lang="ts">
import { useBigPictureMode } from "~/composables/big-picture";
import BigPictureInstallModal from "~/components/BigPictureInstallModal.vue";

const bigPictureState = useBigPictureMode();
const installModalRef = ref();

// Provide the install modal reference to child components
provide('bigPictureInstallModal', installModalRef);
</script>

<style scoped>
/* Hide scrollbars in Big Picture mode */
.scrollbar-hide {
  -ms-overflow-style: none;  /* Internet Explorer 10+ */
  scrollbar-width: none;  /* Firefox */
}

.scrollbar-hide::-webkit-scrollbar {
  display: none;  /* Safari and Chrome */
}
</style>
