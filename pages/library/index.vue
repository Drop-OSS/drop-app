<template>

    <div class="h-full flex flex-col items-center justify-center">
        <div class="text-center">
            <div class="flex flex-col items-center gap-y-4">
                <div class="p-4 rounded-xl bg-zinc-700/50 backdrop-blur-sm">
                    <RocketLaunchIcon class="size-12 text-zinc-400" />
                </div>
                <div>
                    <h3 class="text-xl font-display font-semibold text-zinc-100">Select a game</h3>
                    <p class="mt-1 text-sm text-zinc-400">Choose a game from your library to view details</p>
                </div>
            </div>
        </div>
    </div>
</template>
<script setup lang="ts">
import { RocketLaunchIcon } from '@heroicons/vue/24/outline';
</script>