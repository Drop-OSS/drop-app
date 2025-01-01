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
                    <div class="mt-4 ml-8">
                        <select
                            :value="currentTheme"
                            @change="(e: Event) => handleThemeChange(e)"
                            class="block w-full rounded-md border-0 py-1.5 px-3 bg-zinc-800 text-zinc-100 shadow-sm ring-1 ring-inset ring-zinc-700 focus:ring-2 focus:ring-inset focus:ring-blue-600 sm:text-sm sm:leading-6"
                        >
                            <option v-for="theme in themes" :key="theme.id" :value="theme.id">
                                {{ theme.name }}
                            </option>
                        </select>
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
                            Advanced settings for customizing Drop's appearance and behavior
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
                <div>
                    <h4 class="text-sm font-medium leading-6 text-zinc-100 mb-2">Custom CSS</h4>
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
                    <div class="flex justify-end gap-x-3 mt-3">
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
    </div>
</template>

<script setup lang="ts">
import { SwatchIcon, ChevronRightIcon, CodeBracketIcon } from "@heroicons/vue/24/outline";

const { currentTheme, setTheme, THEMES: themes } = useTheme();
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

function handleThemeChange(event: Event) {
    const select = event.target as HTMLSelectElement;
    setTheme(select.value);
}

// Initialize custom CSS on mount
onMounted(() => {
    if (customCSS.value) {
        applyCustomCSS();
    }
});
</script>
