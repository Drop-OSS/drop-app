import { ref } from 'vue'
import { invoke } from "@tauri-apps/api/core";

// Define valid theme IDs as a const array
const THEME_IDS = [
    'dark',
    'light',
    'theme-purple',
    'theme-green',
    'theme-ocean',
    'theme-amber',
    'theme-rose',
    'theme-indigo'
] as const;

// Create a type from the array values
type ThemeId = typeof THEME_IDS[number];

export interface Theme {
    id: ThemeId;
    name: string;
    bgColor: string;
    palette: string;
}

const THEMES: Theme[] = [
    { id: 'dark', name: 'Dark (Default)', bgColor: 'bg-zinc-950', palette: 'Zinc' },
    { id: 'light', name: 'Light', bgColor: 'bg-white', palette: 'White/Gray' },
    { id: 'theme-purple', name: 'Purple', bgColor: 'bg-purple-900', palette: 'Purple/Violet' },
    { id: 'theme-green', name: 'Green', bgColor: 'bg-emerald-900', palette: 'Emerald/Teal' },
    { id: 'theme-ocean', name: 'Ocean', bgColor: 'bg-slate-900', palette: 'Slate/Cyan' },
    { id: 'theme-amber', name: 'Amber', bgColor: 'bg-amber-900', palette: 'Amber/Orange' },
    { id: 'theme-rose', name: 'Rose', bgColor: 'bg-rose-900', palette: 'Rose/Pink' },
    { id: 'theme-indigo', name: 'Indigo', bgColor: 'bg-indigo-900', palette: 'Indigo/Violet' },
] as const;

const currentTheme = ref<ThemeId>('dark')

export function useTheme() {
    async function setTheme(themeId: string) {
        // Validate theme
        if (!THEME_IDS.includes(themeId as ThemeId)) {
            console.error(`Invalid theme: ${themeId}`);
            return;
        }

        try {
            await invoke('set_theme', { theme: themeId });
            
            // Remove all theme classes
            document.documentElement.classList.remove(...THEME_IDS.filter((t: ThemeId) => t !== 'dark'));
            
            // Add new theme class if it's not the default dark theme
            if (themeId !== 'dark') {
                document.documentElement.classList.add(themeId)
            }
            
            currentTheme.value = themeId as ThemeId
        } catch (error) {
            console.error('Failed to set theme:', error);
        }
    }

    // Initialize theme
    async function initTheme() {
        try {
            const savedTheme = await invoke<string>('get_theme');
            if (THEME_IDS.includes(savedTheme as ThemeId)) {
                await setTheme(savedTheme);
            }
        } catch (error) {
            console.error('Failed to initialize theme:', error);
        }
    }

    return {
        currentTheme,
        setTheme,
        initTheme,
        VALID_THEMES: THEME_IDS,
        THEMES
    }
} 
