<template>
    <div class="divide-y divide-zinc-700">
        <div class="py-6">
            <h2 class="text-base font-semibold font-display leading-7 text-zinc-100">General</h2>
            <p class="mt-1 text-sm leading-6 text-zinc-400">
                Configure basic application settings
            </p>

            <div class="mt-10 space-y-8">
                <div class="flex flex-row items-center justify-between">
                    <div>
                        <h3 class="text-sm font-medium leading-6 text-zinc-100">Start with system</h3>
                        <p class="mt-1 text-sm leading-6 text-zinc-400">
                            Drop will automatically start when you log into your computer
                        </p>
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

                <div class="flex flex-row items-center justify-between">
                    <div>
                        <h3 class="text-sm font-medium leading-6 text-zinc-100">Updates</h3>
                        <p class="mt-1 text-sm leading-6 text-zinc-400">
                            Check for new versions of Drop
                        </p>
                    </div>
                    <div class="flex gap-2">
                        <button
                            @click="checkForUpdates"
                            class="px-3 py-2 text-sm font-semibold text-white bg-blue-600 rounded-md hover:bg-blue-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue-600"
                        >
                            Check for Updates
                        </button>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <UpdateModal
        :is-open="isModalOpen"
        :title="modalTitle"
        :description="modalDescription"
        :button-text="modalButtonText"
        :state="modalState"
        @close="closeModal"
    />
</template>

<script setup lang="ts">
import { Switch } from '@headlessui/vue'
import { invoke } from "@tauri-apps/api/core";
import { ref } from 'vue'
import { listen } from '@tauri-apps/api/event'
import UpdateModal from '~/components/UpdateModal.vue'

defineProps<{}>()

const autostartEnabled = ref<boolean>(false)
const isModalOpen = ref(false)
const modalTitle = ref('')
const modalDescription = ref('')
const modalButtonText = ref('')
const modalState = ref<'checking' | 'up-to-date' | 'update-available' | 'error'>('checking')

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

// update checker
const checkForUpdates = async () => {
    try {
        console.log('Initiating update check from settings page...')
        modalState.value = 'checking'
        modalTitle.value = 'Checking for Updates'
        modalDescription.value = 'Please wait while we check for new versions...'
        modalButtonText.value = 'Close'
        isModalOpen.value = true

        await invoke('check_for_updates')
        
        // The actual update state is handled by the create_modal event listener
        console.log('Update check completed')
    } catch (error) {
        console.error('Failed to check for updates:', error)
        modalState.value = 'error'
        modalTitle.value = 'Update Check Failed'
        modalDescription.value = `Failed to check for updates: ${error}`
        modalButtonText.value = 'Close'
    }
}

const closeModal = () => {
    isModalOpen.value = false
}

// Listen for create_modal events from Rust
onMounted(() => {
    listen('create_modal', (event) => {
        const payload = event.payload as any
        if (payload.type === 'notification') {
            if (payload.data.title === 'Update Available') {
                modalState.value = 'update-available'
            } else {
                modalState.value = 'up-to-date'
            }
            modalTitle.value = payload.data.title
            modalDescription.value = payload.data.description
            modalButtonText.value = payload.data.buttonText
            isModalOpen.value = true
        }
    })
})
</script>
