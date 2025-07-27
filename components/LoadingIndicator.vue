<template></template>

<script setup lang="ts">
const loading = useLoadingIndicator();

watch(loading.isLoading, console.log);
</script>
