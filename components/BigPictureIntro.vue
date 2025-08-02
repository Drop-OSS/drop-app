<template>
  <Transition
    enter-active-class="transition-all duration-300 ease-out"
    enter-from-class="opacity-0 scale-150"
    enter-to-class="opacity-100 scale-100"
    leave-active-class="transition-all duration-300 ease-in"
    leave-from-class="opacity-100 scale-100"
    leave-to-class="opacity-0 scale-150"
  >
    <div
      v-if="showIntro"
      class="fixed inset-0 z-50 flex items-center justify-center bg-zinc-950"
    >
      <!-- Centered Logo Container -->
      <div
        class="relative w-48 h-48 flex items-center justify-center"
      >
        <!-- Drop Logo -->
        <svg 
          class="w-40 h-40 text-blue-400 drop-shadow-2xl" 
          viewBox="0 0 24 24" 
          fill="none" 
          xmlns="http://www.w3.org/2000/svg"
        >
          <path
            d="M4 13.5C4 11.0008 5.38798 8.76189 7.00766 7C8.43926 5.44272 10.0519 4.25811 11.0471 3.5959C11.6287 3.20893 12.3713 3.20893 12.9529 3.5959C13.9481 4.25811 15.5607 5.44272 16.9923 7C18.612 8.76189 20 11.0008 20 13.5C20 17.9183 16.4183 21.5 12 21.5C7.58172 21.5 4 17.9183 4 13.5Z"
            stroke="currentColor" 
            stroke-width="2"
            class="animate-pulse"
          />
        </svg>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
const props = defineProps<{
  show: boolean;
}>();

const showIntro = ref(false);

// Watch for show prop changes
watch(() => props.show, (newShow) => {
  if (newShow) {
    startIntro();
  } else {
    showIntro.value = false;
  }
});

// Start the intro animation sequence
const startIntro = () => {
  showIntro.value = true;
  
  // End intro after duration
  setTimeout(() => {
    showIntro.value = false;
  }, 1500);
};

// Initialize if show is true on mount
onMounted(() => {
  if (props.show) {
    startIntro();
  }
});
</script>