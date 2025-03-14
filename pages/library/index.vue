<template>
  <div class="flex flex-col px-6 py-6 md:px-8 lg:px-10">
    <div class="max-w-2xl">
      <h2 class="text-2xl font-bold font-display text-zinc-100">
        Your Collections
      </h2>
      <p class="mt-2 text-zinc-400">
        Organize your games into collections for easy access.
      </p>
    </div>

    <!-- Collections grid -->
    <TransitionGroup
      name="collection-list"
      tag="div"
      class="mt-8 grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4"
    >
      <!-- Collection buttons -->
      <div
        v-for="collection in collections"
        :key="collection.id"
        class="flex flex-row rounded-lg overflow-hidden transition-all duration-200 text-left w-full hover:scale-105"
      >
        <NuxtLink
          class="grow p-4 bg-zinc-800/50 hover:bg-zinc-800"
          :to="`/library/collection/${collection.id}`"
        >
          <h3 class="text-lg font-semibold text-zinc-100">
            {{ collection.name }}
          </h3>
          <p class="mt-1 text-sm text-zinc-400">
            {{ collection.entries.length }} game(s)
          </p>
        </NuxtLink>

        <!-- Delete button (only for non-default collections) -->
        <button
          v-if="!collection.isDefault"
          @click="() => (currentlyDeleting = collection)"
          class="group px-3 ml-[2px] bg-zinc-800/50 hover:bg-zinc-800"
        >
          <TrashIcon class="transition-all size-5 text-zinc-400 group-hover:text-red-400 group-hover:rotate-[8deg]" />
        </button>
      </div>

      <!-- Create new collection button -->
      <div>
        <button
          @click="collectionCreateOpen = true"
          class="group flex flex-row rounded-lg overflow-hidden transition-all duration-200 text-left w-full hover:scale-105"
        >
          <div class="grow p-4 bg-zinc-800/50 hover:bg-zinc-800 border-2 border-dashed border-zinc-700">
            <div class="flex items-center gap-3">
              <PlusIcon class="h-5 w-5 text-zinc-400 group-hover:text-zinc-300 transition-all duration-300 group-hover:rotate-90" />
              <h3 class="text-lg font-semibold text-zinc-400 group-hover:text-zinc-300">
                Create Collection
              </h3>
            </div>
            <p class="mt-1 text-sm text-zinc-500 group-hover:text-zinc-400">
              Add a new collection to organize your games
            </p>
          </div>
        </button>
      </div>
    </TransitionGroup>
  </div>

  <CreateCollectionModal v-model="collectionCreateOpen" />
  <DeleteCollectionModal 
    v-model="currentlyDeleting" 
  />
</template>

<script setup lang="ts">
import { TrashIcon, PlusIcon } from "@heroicons/vue/20/solid";
import type { Collection } from "~/composables/collections";

const collections = ref(await useCollections());
const collectionCreateOpen = ref(false);
const currentlyDeleting = ref<Collection | undefined>();

async function refreshCollections() {
  collections.value = await useCollections();
}

watch([collectionCreateOpen, currentlyDeleting], async ([createOpen, deleting]) => {
  if (!createOpen && !deleting) {
    await refreshCollections();
  }
});

onMounted(refreshCollections);

useHead({
  title: "Collections",
});
</script>

<style scoped>
.collection-list-enter-active,
.collection-list-leave-active {
  transition: all 0.3s ease;
}

.collection-list-enter-from,
.collection-list-leave-to {
  opacity: 0;
  transform: scale(0.95);
}
</style>
