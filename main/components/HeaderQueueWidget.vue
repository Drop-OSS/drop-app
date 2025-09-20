<script setup lang="ts">
import { ArrowDownTrayIcon } from "@heroicons/vue/20/solid";

const props = defineProps<{ object?: QueueState["queue"][0] }>();
</script>

<template>
  <NuxtLink
    to="/queue"
    class="transition inline-flex items-center cursor-pointer rounded-sm px-4 py-1.5 bg-zinc-900 hover:bg-zinc-800 relative"
  >
    <ArrowDownTrayIcon
      :class="[
        'h-5 z-50',
        props.object
          ? 'text-white hover:text-zinc-300'
          : 'text-zinc-600 hover:text-zinc-300',
      ]"
    />
    <div
      v-if="props.object?.progress"
      :class="[
        'transition-all duration-500 absolute left-0 top-0 bottom-0 z-10',
        props.object.status === 'Validating' ? 'bg-green-600' : 'bg-blue-600'
      ]"
      :style="{ width: `${props.object.progress * 99 + 1}%` }"
    />
  </NuxtLink>
</template>
