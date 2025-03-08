<template>
  <TransitionRoot appear :show="isOpen" as="template">
    <Dialog as="div" @close="closeModal" class="relative z-50">
      <TransitionChild as="template" enter="duration-300 ease-out" enter-from="opacity-0" enter-to="opacity-100"
        leave="duration-200 ease-in" leave-from="opacity-100" leave-to="opacity-0">
        <div class="fixed inset-0 bg-black/25" />
      </TransitionChild>

      <div class="fixed inset-0 overflow-y-auto">
        <div class="flex min-h-full items-center justify-center p-4">
          <TransitionChild as="template" enter="duration-300 ease-out" enter-from="opacity-0 scale-95"
            enter-to="opacity-100 scale-100" leave="duration-200 ease-in" leave-from="opacity-100 scale-100"
            leave-to="opacity-0 scale-95">
            <DialogPanel
              class="w-full max-w-md transform overflow-hidden rounded-2xl bg-zinc-800 p-6 text-left align-middle shadow-xl transition-all">
              
              <!-- Loading State -->
              <div v-if="state === 'checking'" class="text-center py-4">
                <div class="animate-spin inline-block w-10 h-10 border-4 border-blue-600 border-t-transparent rounded-full mb-4"></div>
                <DialogTitle as="h3" class="text-lg font-medium leading-6 text-zinc-100">
                  Checking for Updates
                </DialogTitle>
                <p class="mt-2 text-sm text-zinc-400">
                  Please wait while we check for new versions...
                </p>
              </div>

              <!-- Up to Date State -->
              <div v-else-if="state === 'up-to-date'" class="text-center py-4">
                <div class="inline-block w-12 h-12 rounded-full bg-green-600/10 p-2 mb-4">
                  <CheckCircleIcon class="w-8 h-8 text-green-600" />
                </div>
                <DialogTitle as="h3" class="text-lg font-medium leading-6 text-zinc-100">
                  You're Up to Date!
                </DialogTitle>
                <p class="mt-2 text-sm text-zinc-400">
                  {{ description }}
                </p>
                <div class="mt-6">
                  <button @click="closeModal"
                    class="inline-flex justify-center rounded-md bg-green-600 px-4 py-2 text-sm font-medium text-white hover:bg-green-500 focus:outline-none focus-visible:ring-2 focus-visible:ring-green-500 focus-visible:ring-offset-2">
                    {{ buttonText }}
                  </button>
                </div>
              </div>

              <!-- Update Available State -->
              <div v-else-if="state === 'update-available'" class="text-center py-4">
                <div class="inline-block w-12 h-12 rounded-full bg-blue-600/10 p-2 mb-4">
                  <ArrowUpCircleIcon class="w-8 h-8 text-blue-600" />
                </div>
                <DialogTitle as="h3" class="text-lg font-medium leading-6 text-zinc-100">
                  {{ title }}
                </DialogTitle>
                <p class="mt-2 text-sm text-zinc-400">
                  A new version of Drop ({{ getVersion(description) }}) is available. 
                  <a 
                    href="#" 
                    @click.prevent="openGitHubReleases" 
                    class="text-blue-400 hover:text-blue-300 underline"
                  >
                    Click here to visit the releases page
                  </a>
                </p>
                <div class="mt-6">
                  <button @click="closeModal"
                    class="inline-flex justify-center rounded-md bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-500 focus:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 focus-visible:ring-offset-2">
                    {{ buttonText }}
                  </button>
                </div>
              </div>

              <!-- Error State -->
              <div v-else-if="state === 'error'" class="text-center py-4">
                <div class="inline-block w-12 h-12 rounded-full bg-red-600/10 p-2 mb-4">
                  <ExclamationCircleIcon class="w-8 h-8 text-red-600" />
                </div>
                <DialogTitle as="h3" class="text-lg font-medium leading-6 text-zinc-100">
                  {{ title }}
                </DialogTitle>
                <p class="mt-2 text-sm text-zinc-400">
                  {{ description }}
                </p>
                <div class="mt-6">
                  <button @click="closeModal"
                    class="inline-flex justify-center rounded-md bg-red-600 px-4 py-2 text-sm font-medium text-white hover:bg-red-500 focus:outline-none focus-visible:ring-2 focus-visible:ring-red-500 focus-visible:ring-offset-2">
                    {{ buttonText }}
                  </button>
                </div>
              </div>

            </DialogPanel>
          </TransitionChild>
        </div>
      </div>
    </Dialog>
  </TransitionRoot>
</template>

<script setup lang="ts">
import {
  Dialog,
  DialogPanel,
  DialogTitle,
  TransitionChild,
  TransitionRoot,
} from '@headlessui/vue'
import {
  CheckCircleIcon,
  ArrowUpCircleIcon,
  ExclamationCircleIcon
} from '@heroicons/vue/24/outline'
import { open } from '@tauri-apps/plugin-shell'

const props = defineProps<{
  title: string
  description: string
  buttonText: string
  isOpen: boolean
  state: 'checking' | 'up-to-date' | 'update-available' | 'error'
}>()

const emit = defineEmits<{
  (e: 'close'): void
}>()

const closeModal = () => {
  emit('close')
}

const getVersion = (description: string) => {
  const match = description.match(/Version (.*?) is/)
  return match ? match[1] : description
}

// Open GitHub releases page in browser
const openGitHubReleases = async () => {
  try {
    await open('https://github.com/Drop-OSS/drop-app/releases')
  } catch (error) {
    console.error('Failed to open GitHub releases:', error)
  }
}
</script> 
