<template>
  <div class="mx-auto max-w-7xl px-8">
    <div class="border-b border-zinc-700 py-5">
      <h3 class="text-base font-semibold font-display leading-6 text-zinc-100">
        Account
      </h3>
    </div>

    <div class="mt-6">
      <dl class="divide-y divide-zinc-800">
        <div class="py-4 sm:grid sm:grid-cols-3 sm:gap-4">
          <dt class="text-sm font-medium text-zinc-400">Username</dt>
          <dd class="mt-1 text-sm text-zinc-100 sm:col-span-2 sm:mt-0">{{ state.user?.displayName }}</dd>
        </div>
        <div class="py-4 sm:grid sm:grid-cols-3 sm:gap-4">
          <dt class="text-sm font-medium text-zinc-400">Client ID</dt>
          <dd class="mt-1 text-sm text-zinc-100 sm:col-span-2 sm:mt-0">{{ accountInfo?.client_id || 'Loading...' }}</dd>
        </div>
      </dl>

      <div class="mt-6">
        <button
          @click="logout"
          type="button"
          class="inline-flex items-center gap-x-2 rounded-md bg-red-600 px-3.5 py-2.5 text-sm font-semibold text-white shadow-sm hover:bg-red-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-red-600"
        >
          <ArrowRightOnRectangleIcon class="-ml-0.5 size-5" aria-hidden="true" />
          Logout
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { ArrowRightOnRectangleIcon } from '@heroicons/vue/20/solid';
import { invoke } from '@tauri-apps/api/core';
import { useAppState } from "~/composables/app-state";

interface AccountInfo {
  client_id: string;
}

const state = useAppState();
const accountInfo = ref<AccountInfo>();

onMounted(async () => {
  try {
    accountInfo.value = await invoke<AccountInfo>('fetch_account_info');
  } catch (error) {
    console.error('Failed to fetch account info:', error);
  }
});

async function logout() {
  // We'll implement this next
  console.log('Logout clicked');
}
</script>