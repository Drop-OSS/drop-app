<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { ref } from 'vue';

const state = ref({});

invoke("fetch_state").then((value) => {
    state.value = value as any;
})
</script>

<template>
    hello world
  <Suspense>
    {{ state }}
  </Suspense>
</template>

<style scoped></style>
