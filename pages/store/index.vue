<template>
  <div class="grow w-full h-full flex items-center justify-center">
    <div class="flex flex-col items-center">
      <BuildingStorefrontIcon
        class="h-12 w-12 text-blue-600"
        aria-hidden="true"
      />
      <div class="mt-3 text-center sm:mt-5">
        <h1 class="text-3xl font-semibold font-display leading-6 text-zinc-100">
          Store not supported in client
        </h1>
        <div class="mt-4">
          <p class="text-sm text-zinc-400 max-w-lg">
            Currently, Drop requires you to view the store in your browser.
            Please click the button below to open it in your default browser.
          </p>
          <NuxtLink
            :href="storeUrl"
            target="_blank"
            class="mt-6 transition text-sm/6 font-semibold text-zinc-400 hover:text-zinc-100 inline-flex gap-x-2 items-center duration-200 hover:scale-105"
          >
            Open Store <ArrowTopRightOnSquareIcon class="size-4" />
          </NuxtLink>
        </div>
      </div>
    </div>
  </div>
</template>
<script setup lang="ts">
import {
  ArrowTopRightOnSquareIcon,
  BuildingStorefrontIcon,
} from "@heroicons/vue/20/solid";
import { invoke } from "@tauri-apps/api/core";

const storeUrl = await invoke<string>("gen_drop_url", { path: "/store" });
</script>
