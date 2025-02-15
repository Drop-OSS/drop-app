<template>
  <div class="flex flex-row h-full">
    <!-- Sidebar -->
    <div
      class="flex-none max-h-full overflow-y-auto w-72 bg-zinc-950/50 backdrop-blur-xl px-4 py-3 border-r border-zinc-800/50"
    >
      <LibrarySearch />
    </div>
    <div class="grow overflow-y-auto">
      <NuxtPage :libraryDownloadError="libraryDownloadError" />
    </div>
  </div>
</template>

<script setup lang="ts">
let libraryDownloadError = false;
</script>

<style scoped>
.list-move,
.list-enter-active,
.list-leave-active {
  transition: all 0.3s ease;
}

.list-enter-from,
.list-leave-to {
  opacity: 0;
  transform: translateX(-30px);
}

.list-leave-active {
  position: absolute;
}
</style>
