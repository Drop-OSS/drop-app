<template>
  <div class="flex flex-col bg-zinc-900 overflow-hidden">
    <Header class="select-none" />
    <div class="grow overflow-y-auto">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
const queueState = useQueueState();
</script>
