import { ref } from 'vue'

const currentTheme = ref('dark')

export function useTheme() {
    function setTheme(themeId: string) {
        // Remove all theme classes
        document.documentElement.classList.remove('light', 'theme-purple', 'theme-green', 'theme-ocean', 'theme-amber', 'theme-rose', 'theme-indigo')
        
        // Add new theme class if it's not the default dark theme
        if (themeId !== 'dark') {
            document.documentElement.classList.add(themeId)
        }
        
        currentTheme.value = themeId
        localStorage.setItem('theme', themeId)
    }

    // Initialize theme
    function initTheme() {
        const savedTheme = localStorage.getItem('theme')
        if (savedTheme) {
            setTheme(savedTheme)
        }
    }

    return {
        currentTheme,
        setTheme,
        initTheme
    }
} 