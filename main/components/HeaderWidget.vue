<template>
  <div
    :class="[
      'transition inline-flex items-center cursor-pointer rounded-sm px-4 py-1.5  text-zinc-600  hover:text-zinc-300 relative',
      props.notifications !== undefined
        ? 'bg-blue-400'
        : props.problem !== undefined && props.problem
          ? 'bg-red-400'
          : 'bg-zinc-900 hover:bg-zinc-800',
    ]"
  >
    <slot />
    <div
      v-if="props.notifications !== undefined"
      class="text-zinc-900 absolute top-0 right-0 translate-x-[30%] translate-y-[-30%] text-xs bg-blue-400 rounded-full w-3.5 h-3.5 text-center"
    >
      {{ props.notifications }}
    </div>
    <div
      v-else-if="props.problem !== undefined && props.problem"
      class="text-zinc-100 absolute top-0 right-0 translate-x-[30%] translate-y-[-30%] text-sm bg-red-400 rounded-full w-5 h-5 text-center"
    >
      !
    </div>
  </div>
</template>

<script setup lang="ts">
const props = defineProps<{
  notifications?: number;
  problem?: boolean;
  class?: string;
}>();
</script>
