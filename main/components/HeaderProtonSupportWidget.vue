<template>
  <NuxtLink to="/settings/compat">
    <HeaderWidget
      v-if="appState && appState.umuState !== 'NotNeeded'"
      :problem="protonError"
    >
      <img src="/proton-logo.png" class="relative z-50 size-5 brightness-[30%]" />
    </HeaderWidget>
  </NuxtLink>
</template>

<script setup lang="ts">
const appState = useAppState();
const paths = await useProtonPaths();

const protonError = computed(() => appState.value?.umuState === "NotInstalled" || !paths.data.value.default);
</script>
