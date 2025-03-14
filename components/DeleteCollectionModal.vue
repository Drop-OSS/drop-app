<template>
  <ModalTemplate :model-value="!!collection" @update:model-value="updateValue">
    <template #default>
      <div>
        <DialogTitle as="h3" class="text-lg font-bold font-display text-zinc-100">
          Delete Collection
        </DialogTitle>
        <p class="mt-1 text-sm text-zinc-400">
          Are you sure you want to delete "{{ collection?.name }}"?
        </p>
        <p class="mt-2 text-sm font-bold text-red-500">
          This action cannot be undone.
        </p>
      </div>
    </template>

    <template #buttons>
      <LoadingButton
        :loading="deleteLoading"
        @click="handleDelete"
        class="bg-red-600 text-white hover:bg-red-500"
      >
        Delete
      </LoadingButton>
      <button
        @click="closeModal"
        class="inline-flex items-center rounded-md bg-zinc-800 px-3 py-2 text-sm font-semibold font-display text-white hover:bg-zinc-700"
      >
        Cancel
      </button>
    </template>
  </ModalTemplate>
</template>

<script setup lang="ts">
import { DialogTitle } from "@headlessui/vue";
import type { Collection } from "~/composables/collections";
import { deleteCollection } from "~/composables/collections";
import { useRouter } from "vue-router";

const collection = defineModel<Collection | undefined>();
const deleteLoading = ref(false);
const router = useRouter();

async function handleDelete() {
  if (!collection.value) return;

  try {
    deleteLoading.value = true;
    const id = collection.value.id; // Store ID before clearing collection
    collection.value = undefined; // Close modal immediately
    await deleteCollection(id); // Delete using stored ID
    await refreshNuxtData();
    router.push('/library');
  } catch (error) {
    console.error("Failed to delete collection:", error);
  } finally {
    deleteLoading.value = false;
  }
}

function updateValue(value: boolean | undefined) {
  if (!value) {
    collection.value = undefined;
  }
}

function closeModal() {
  collection.value = undefined;
}
</script> 
