import { invoke } from "@tauri-apps/api/core";
import type { Component } from "vue";

export interface BigPictureNavItem {
  icon: Component;
  label: string;
  route: string;
  description: string;
  notifications?: number;
}

export interface BigPictureState {
  isActive: boolean;
  currentPage: string;
  lastNormalPage: string;
  showIntro: boolean;
}

export const useBigPictureMode = () => useState<BigPictureState>("big-picture", () => ({
  isActive: false,
  currentPage: "/big-picture/library",
  lastNormalPage: "/library",
  showIntro: false
}));

export const toggleBigPictureMode = async () => {
  const state = useBigPictureMode();
  
  if (state.value.isActive) {
    await exitBigPictureMode();
  } else {
    await enterBigPictureMode();
  }
};

export const enterBigPictureMode = async () => {
  const state = useBigPictureMode();
  const router = useRouter();
  
  try {
    // Store current page before entering big picture mode
    state.value.lastNormalPage = router.currentRoute.value.path;
    
    // Enter fullscreen
    await invoke("enter_fullscreen");
    
    // Set big picture mode active and show intro
    state.value.isActive = true;
    state.value.showIntro = true;
    
    // Navigate to big picture library
    await router.push("/big-picture/library");
    
    // Hide intro after animation completes
    setTimeout(() => {
      state.value.showIntro = false;
    }, 1800); // Slightly longer than the intro animation
    
  } catch (error) {
    console.error("Failed to enter big picture mode:", error);
  }
};

export const exitBigPictureMode = async () => {
  const state = useBigPictureMode();
  const router = useRouter();
  
  try {
    // Exit fullscreen
    await invoke("exit_fullscreen");
    
    // Set big picture mode inactive
    state.value.isActive = false;
    
    // Navigate back to last normal page
    await router.push(state.value.lastNormalPage);
    
  } catch (error) {
    console.error("Failed to exit big picture mode:", error);
  }
};

export const setBigPicturePage = (page: string) => {
  const state = useBigPictureMode();
  state.value.currentPage = page;
}; 