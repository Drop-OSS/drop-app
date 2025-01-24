<script setup lang="ts">
const props = defineProps<{ libraryDownloadError: boolean }>();
</script>
<template>
    <div v-if="libraryDownloadError" class="mx-auto pt-10 text-center text-gray-500">
        Library Failed to update
    </div>

</template>