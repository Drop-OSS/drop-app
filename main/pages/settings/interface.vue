<template>
  <div class="grow w-full h-full">
    <div class="border-b border-zinc-700 py-5">
      <h3 class="text-base font-semibold font-display leading-6 text-zinc-100">
        Interface
      </h3>
    </div>

    <div class="mt-5">
      <div class="space-y-8">
        <div class="flex flex-row items-center justify-between">
          <div>
            <h3 class="text-sm font-medium leading-6 text-zinc-100">Library View</h3>
            <p class="mt-1 text-sm leading-6 text-zinc-400">
              Choose how games are displayed in your library
            </p>
          </div>
          <div class="flex gap-x-2">
            <button
              @click="() => updateLibraryView('list')"
              :class="[
                'px-4 py-2 rounded-lg text-sm font-semibold transition-all duration-200 transform hover:scale-105 active:scale-95 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:ring-offset-zinc-900',
                libraryView === 'list'
                  ? 'bg-blue-600 text-white'
                  : 'bg-zinc-800 text-zinc-300 hover:bg-zinc-700',
              ]"
            >
              List
            </button>
            <button
              @click="() => updateLibraryView('grid')"
              :class="[
                'px-4 py-2 rounded-lg text-sm font-semibold transition-all duration-200 transform hover:scale-105 active:scale-95 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:ring-offset-zinc-900',
                libraryView === 'grid'
                  ? 'bg-blue-600 text-white'
                  : 'bg-zinc-800 text-zinc-300 hover:bg-zinc-700',
              ]"
            >
              Grid
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";

const { libraryView, toggleView } = useLibraryView();

async function updateLibraryView(view: "list" | "grid") {
  if (libraryView.value !== view) {
    libraryView.value = view;
    await invoke("update_settings", {
      newSettings: { libraryView: view },
    });
  }
}
</script>
