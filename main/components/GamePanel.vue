<template>
  <NuxtLink
    :href="href"
    :class="[
      'group relative flex-1 min-w-42 max-w-48 h-64 rounded-lg overflow-hidden',
      'transition-all duration-300 text-left hover:scale-[1.02] hover:shadow-lg hover:-translate-y-0.5',
      'focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:ring-offset-zinc-900',
      isSelected
        ? 'ring-2 ring-blue-500 shadow-lg shadow-blue-500/20'
        : '',
    ]"
  >
    <!-- Background Image -->
    <div
      :class="{
        'transition-all duration-300 group-hover:scale-110': true,
      }"
      class="absolute inset-0"
    >
      <!-- Cover Image (if available) -->
      <img
        v-if="coverSrc"
        :src="coverSrc"
        :alt="gameName"
        class="w-full h-full object-cover brightness-[90%] blur-[1px]"
      />
      <!-- Fallback to Icon -->
      <div
        v-else-if="iconSrc"
        class="w-full h-full flex items-center justify-center bg-gradient-to-br from-zinc-800 to-zinc-900"
      >
        <img
          class="w-2/3 h-2/3 object-contain brightness-[90%] blur-[1px]"
          :src="iconSrc"
          :alt="gameName"
        />
      </div>
      <!-- Placeholder -->
      <div
        v-else
        class="w-full h-full bg-gradient-to-br from-zinc-800 to-zinc-900"
      />
      <!-- Gradient Overlay -->
      <div
        class="absolute inset-0 bg-gradient-to-t from-zinc-950/80 via-zinc-950/0 to-transparent"
      />
    </div>

    <!-- Content Overlay -->
    <div class="absolute bottom-0 left-0 w-full p-3">
      <h1
        :class="{
          'group-hover:text-white transition-colors': true,
        }"
        class="text-zinc-100 text-sm font-bold font-display mb-1"
      >
        {{ gameName }}
      </h1>
      <p
        v-if="description"
        :class="{
          'group-hover:text-zinc-300 transition-colors': true,
        }"
        class="text-zinc-400 text-xs line-clamp-2 mb-1.5"
      >
        {{ description }}
      </p>
      <!-- Status Badge -->
      <div v-if="statusText" class="flex items-center mt-1">
        <span
          :class="[
            'inline-flex items-center px-2 py-0.5 rounded-md text-[10px] font-bold uppercase font-display',
            statusClass,
          ]"
        >
          {{ statusText }}
        </span>
      </div>
    </div>

    <!-- Selected Indicator -->
    <div
      v-if="isSelected"
      class="absolute top-3 right-3 z-20 w-2.5 h-2.5 rounded-full bg-blue-500 shadow-lg shadow-blue-500/50 ring-2 ring-blue-500/30"
    />
  </NuxtLink>
</template>

<script setup lang="ts">
defineProps<{
  href: string;
  gameName: string;
  description?: string;
  coverSrc?: string;
  iconSrc?: string;
  statusText?: string;
  statusClass?: string;
  isSelected?: boolean;
}>();
</script>
