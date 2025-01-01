<template>
    <div class="divide-y divide-zinc-700">
        <div class="py-6">
            <h2 class="text-base font-semibold font-display leading-7 text-zinc-100">Interface Settings</h2>
            <p class="mt-1 text-sm leading-6 text-zinc-400">
                Customize the appearance of Drop
            </p>

            <div class="mt-10 space-y-8">
                <div>
                    <div class="flex items-center gap-x-3">
                        <SwatchIcon class="h-5 w-5 text-zinc-400" />
                        <div>
                            <h3 class="text-sm font-medium leading-6 text-zinc-100">Theme</h3>
                            <p class="mt-1 text-sm leading-6 text-zinc-400">
                                Choose your preferred color theme
                            </p>
                        </div>
                    </div>
                    <div class="mt-4 ml-8 grid grid-cols-2 gap-3 sm:grid-cols-4">
                        <button
                            v-for="theme in themes"
                            :key="theme.id"
                            @click="setTheme(theme.id)"
                            :class="[
                                currentTheme === theme.id ? 'ring-2 ring-offset-2 ring-offset-zinc-900 ring-blue-500' : '',
                                'relative flex flex-col items-center justify-center rounded-lg p-3 focus:outline-none'
                            ]"
                        >
                            <span :class="theme.bgColor" class="h-8 w-8 rounded-full mb-2" />
                            <span class="text-xs text-zinc-100">{{ theme.name }}</span>
                            <span class="text-xs text-zinc-400">{{ theme.palette }}</span>
                        </button>
                    </div>
                </div>
            </div>
        </div>
        
        <div class="py-6">
            <div class="flex items-center gap-x-3">
                <CodeBracketIcon class="h-5 w-5 text-zinc-400" />
                <div class="flex items-center gap-x-2">
                    <div>
                        <h3 class="text-sm font-medium leading-6 text-zinc-100">Advanced Customization</h3>
                        <p class="mt-1 text-sm leading-6 text-zinc-400">
                            Add custom CSS styles to override the default theme
                        </p>
                    </div>
                    <button 
                        @click="showAdvanced = !showAdvanced"
                        class="text-zinc-400 hover:text-zinc-200 transition-colors"
                    >
                        <ChevronRightIcon 
                            class="h-5 w-5 transition-transform duration-200"
                            :class="{ 'rotate-90': showAdvanced }"
                        />
                    </button>
                </div>
            </div>

            <div v-if="showAdvanced" class="mt-4 ml-8 space-y-4">
                <div class="relative">
                    <textarea
                        v-model="customCSS"
                        rows="10"
                        class="w-full rounded-md bg-zinc-800/50 border-0 px-4 py-3 text-sm text-zinc-200 font-mono shadow-sm ring-1 ring-inset ring-zinc-700 focus:ring-2 focus:ring-inset focus:ring-blue-500"
                        placeholder="/* Add your custom CSS here */
:root {
    --color-accent: theme('colors.pink.500');
    --color-accent-hover: theme('colors.pink.400');
}"
                    ></textarea>
                </div>
                <div class="flex justify-end gap-x-3">
                    <button
                        @click="resetCustomCSS"
                        class="rounded-md bg-zinc-800 px-3 py-2 text-sm font-semibold text-zinc-100 shadow-sm hover:bg-zinc-700"
                    >
                        Reset
                    </button>
                    <button
                        @click="applyCustomCSS"
                        class="rounded-md bg-blue-600 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-blue-500"
                    >
                        Apply
                    </button>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { SwatchIcon, ChevronRightIcon, CodeBracketIcon } from "@heroicons/vue/24/outline";

const themes = [
    { id: 'dark', name: 'Dark (Default)', bgColor: 'bg-zinc-950', palette: 'Zinc' },
    { id: 'light', name: 'Light', bgColor: 'bg-white', palette: 'White/Gray' },
    { id: 'theme-purple', name: 'Purple', bgColor: 'bg-purple-900', palette: 'Purple/Violet' },
    { id: 'theme-green', name: 'Green', bgColor: 'bg-emerald-900', palette: 'Emerald/Teal' },
    { id: 'theme-ocean', name: 'Ocean', bgColor: 'bg-slate-900', palette: 'Slate/Cyan' },
    { id: 'theme-amber', name: 'Amber', bgColor: 'bg-amber-900', palette: 'Amber/Orange' },
    { id: 'theme-rose', name: 'Rose', bgColor: 'bg-rose-900', palette: 'Rose/Pink' },
    { id: 'theme-indigo', name: 'Indigo', bgColor: 'bg-indigo-900', palette: 'Indigo/Violet' },
];

const { currentTheme, setTheme } = useTheme();
const showAdvanced = ref(false);
const customCSS = ref(localStorage.getItem('customCSS') || '');

// Create style element for custom CSS
const customStyleElement = ref<HTMLStyleElement | null>(null);

function applyCustomCSS() {
    // Store in localStorage
    localStorage.setItem('customCSS', customCSS.value);
    
    // Create or update style element
    if (!customStyleElement.value) {
        customStyleElement.value = document.createElement('style');
        customStyleElement.value.id = 'custom-css';
        document.head.appendChild(customStyleElement.value);
    }
    customStyleElement.value.textContent = customCSS.value;
}

function resetCustomCSS() {
    customCSS.value = '';
    localStorage.removeItem('customCSS');
    if (customStyleElement.value) {
        customStyleElement.value.textContent = '';
    }
}

// Initialize custom CSS on mount
onMounted(() => {
    if (customCSS.value) {
        applyCustomCSS();
    }
});
</script>
