<template>
  <div tvnav-id="mainbody" class="flex flex-row bg-zinc-900 overflow-hidden h-screen">
    <Sidebar class="select-none" />
    <div class="relative grow overflow-y-auto">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
const queueState = useQueueState();
</script>
