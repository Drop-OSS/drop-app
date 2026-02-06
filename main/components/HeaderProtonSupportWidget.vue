<template>
  <NuxtLink to="/settings/compat">
    <HeaderWidget
      v-if="appState && appState.umuState !== 'NotNeeded'"
      :problem="notInstalled"
    >
      <img src="/proton-logo.png" class="relative z-50 size-5 brightness-[30%]" />
    </HeaderWidget>
  </NuxtLink>
</template>

<script setup lang="ts">
const appState = useAppState();

const notInstalled = appState.value?.umuState === "NotInstalled";
</script>
