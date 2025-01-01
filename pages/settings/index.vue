<template>
    <div class="divide-y divide-zinc-700">
        <div class="py-6">
            <h2 class="text-base font-semibold font-display leading-7 text-zinc-100">General</h2>
            <p class="mt-1 text-sm leading-6 text-zinc-400">
                Configure basic application settings
            </p>

            <div class="mt-10 space-y-8">
                <div class="flex flex-row items-center justify-between">
                    <div class="flex items-center gap-x-3">
                        <PlayIcon class="h-5 w-5 text-zinc-400" />
                        <div>
                            <h3 class="text-sm font-medium leading-6 text-zinc-100">Start with system</h3>
                            <p class="mt-1 text-sm leading-6 text-zinc-400">
                                Drop will automatically start when you log into your computer
                            </p>
                        </div>
                    </div>
                    <Switch
                        v-model="autostartEnabled"
                        :class="[
                            autostartEnabled ? 'bg-blue-600' : 'bg-zinc-700',
                            'relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out'
                        ]"
                    >
                        <span
                            :class="[
                                autostartEnabled ? 'translate-x-5' : 'translate-x-0',
                                'pointer-events-none relative inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out'
                            ]"
                        />
                    </Switch>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { Switch } from '@headlessui/vue'
import { invoke } from "@tauri-apps/api/core";
import { PlayIcon } from "@heroicons/vue/24/outline";

const autostartEnabled = ref<boolean>(false)

// Load initial state
invoke('get_autostart_enabled').then((enabled) => {
    autostartEnabled.value = enabled as boolean
})

// Watch for changes and update autostart
watch(autostartEnabled, async (newValue: boolean) => {
    try {
        await invoke('toggle_autostart', { enabled: newValue })
    } catch (error) {
        console.error('Failed to toggle autostart:', error)
        // Revert the toggle if it failed
        autostartEnabled.value = !newValue
    }
})
</script>
