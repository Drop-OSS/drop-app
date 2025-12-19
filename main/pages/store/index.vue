<template>
  <iframe :src="convertedStoreUrl" class="grow w-full h-full" />
</template>
<script setup lang="ts">
import {
  ArrowTopRightOnSquareIcon,
  BuildingStorefrontIcon,
} from "@heroicons/vue/20/solid";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";

const convertedStoreUrl = convertFileSrc("store", "server");
</script>
