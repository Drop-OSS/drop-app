<template>
  <ModalTemplate :model-value="open" @update:model-value="open = $event">
    <template #default>
      <div>
        <DialogTitle as="h3" class="text-lg font-medium leading-6 text-white">
          Create collection
        </DialogTitle>
        <p class="mt-1 text-zinc-400 text-sm">
          Collections can be used to organize your games and find them more easily,
          especially if you have a large library.
        </p>
      </div>
      <div class="mt-2">
        <form @submit.prevent="handleCreate">
          <input
            type="text"
            v-model="collectionName"
            placeholder="Collection name"
            class="block w-full rounded-md border-0 bg-zinc-800 py-1.5 text-white shadow-sm ring-1 ring-inset ring-zinc-700 placeholder:text-zinc-400 focus:ring-2 focus:ring-inset focus:ring-blue-600 sm:text-sm sm:leading-6"
          />
          <button class="hidden" type="submit" />
        </form>
      </div>
    </template>

    <template #buttons="{ close }">
      <LoadingButton
        :loading="createLoading"
        :disabled="!collectionName"
        @click="handleCreate"
        class="w-full sm:w-fit"
      >
        Create
      </LoadingButton>
      <button
        type="button"
        class="mt-3 inline-flex w-full justify-center rounded-md bg-zinc-800 px-3 py-2 text-sm font-semibold text-zinc-100 shadow-sm ring-1 ring-inset ring-zinc-800 hover:bg-zinc-900 sm:mt-0 sm:w-auto"
        @click="close"
      >
        Cancel
      </button>
    </template>
  </ModalTemplate>
</template>

<script setup lang="ts">
import { DialogTitle } from "@headlessui/vue";
import { createCollection } from "~/composables/collections";
import { useRouter } from 'vue-router';

const open = defineModel<boolean>();
const collectionName = ref("");
const createLoading = ref(false);
const router = useRouter();

async function handleCreate() {
  if (!collectionName.value || createLoading.value) return;

  try {
    createLoading.value = true;
    await createCollection(collectionName.value);
    collectionName.value = "";
    open.value = false;
    // Refresh the collections data
    await refreshNuxtData();
  } catch (error) {
    console.error("Failed to create collection:", error);
  } finally {
    createLoading.value = false;
  }
}
</script> 
