<template>
    <div v-if="libraryDownloadError" class="mx-auto pt-10 text-center text-gray-500">
        Library Failed to update
    </div>

    <div class="h-full flex flex-col items-center justify-center">
        <div class="text-center">
            <div class="flex flex-col items-center gap-y-4">
                <div class="p-4 rounded-xl bg-zinc-800/50 backdrop-blur-sm">
                    <RocketLaunchIcon class="size-12 text-zinc-600" />
                </div>
                <div>
                    <h3 class="text-xl font-display font-semibold text-zinc-100">Select a game</h3>
                    <p class="mt-1 text-sm text-zinc-500">Choose a game from your library to view details</p>
                </div>
            </div>
        </div>
    </div>
</template>
<script setup lang="ts">
import { RocketLaunchIcon } from '@heroicons/vue/24/outline';

const props = defineProps<{ libraryDownloadError: boolean }>();
</script>