<template>
    <div class="divide-y divide-zinc-700">
        <div class="py-6">
            <h2 class="text-base font-semibold font-display leading-7 text-zinc-100">Interface Settings</h2>
            <p class="mt-1 text-sm leading-6 text-zinc-400">
                Customize the appearance of Drop
            </p>

            <div class="mt-10 space-y-8">
                <div class="flex flex-row items-center justify-between">
                    <div class="flex items-center gap-x-3">
                        <SunIcon class="h-5 w-5 text-zinc-400" />
                        <div>
                            <h3 class="text-sm font-medium leading-6 text-zinc-100">Light mode</h3>
                            <p class="mt-1 text-sm leading-6 text-zinc-400">
                                Enable light color theme
                            </p>
                        </div>
                    </div>
                    <Switch
                        v-model="lightMode"
                        :class="[
                            lightMode ? 'bg-blue-600' : 'bg-zinc-700',
                            'relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out'
                        ]"
                    >
                        <span
                            :class="[
                                lightMode ? 'translate-x-5' : 'translate-x-0',
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
import { SunIcon } from "@heroicons/vue/24/outline";

const lightMode = ref<boolean>(false)

// Watch for light mode changes
watch(lightMode, (newValue: boolean) => {
    document.documentElement.classList.toggle('light', newValue)
    localStorage.setItem('theme', newValue ? 'light' : 'dark')
})

// Initialize theme from localStorage
onMounted(() => {
    const savedTheme = localStorage.getItem('theme')
    if (savedTheme) {
        lightMode.value = savedTheme === 'light'
        document.documentElement.classList.toggle('light', lightMode.value)
    }
})
</script>
