<template>
  <div class="divide-y divide-zinc-700">
    <div class="py-6">
      <h2 class="text-base font-semibold font-display leading-7 text-zinc-100">Debug Information</h2>
      <p class="mt-1 text-sm leading-6 text-zinc-400">
        Technical information about your Drop client installation, helpful for debugging.
      </p>

      <div class="mt-10 space-y-8">
        <div>
          <div class="flex items-center gap-x-3">
            <FingerPrintIcon class="h-5 w-5 text-zinc-400" />
            <h3 class="text-sm font-medium leading-6 text-zinc-100">Client ID</h3>
          </div>
          <p class="mt-2 text-sm text-zinc-400 font-mono ml-8">
            {{ clientId || 'Not signed in' }}
          </p>
        </div>

        <div>
          <div class="flex items-center gap-x-3">
            <TagIcon class="h-5 w-5 text-zinc-400" />
            <h3 class="text-sm font-medium leading-6 text-zinc-100">Client Version</h3>
          </div>
          <p class="mt-2 text-sm text-zinc-400 font-mono ml-8">
            0.1.0
          </p>
        </div>

        <div>
          <div class="flex items-center gap-x-3">
            <ComputerDesktopIcon class="h-5 w-5 text-zinc-400" />
            <h3 class="text-sm font-medium leading-6 text-zinc-100">Platform</h3>
          </div>
          <p class="mt-2 text-sm text-zinc-400 font-mono ml-8">
            {{ platformInfo }}
          </p>
        </div>

        <div>
          <div class="flex items-center gap-x-3">
            <ServerIcon class="h-5 w-5 text-zinc-400" />
            <h3 class="text-sm font-medium leading-6 text-zinc-100">Server URL</h3>
          </div>
          <p class="mt-2 text-sm text-zinc-400 font-mono ml-8">
            {{ serverUrl || 'Not connected' }}
          </p>
        </div>

        <div>
          <div class="flex items-center gap-x-3">
            <FolderIcon class="h-5 w-5 text-zinc-400" />
            <h3 class="text-sm font-medium leading-6 text-zinc-100">Data Directory</h3>
          </div>
          <p class="mt-2 text-sm text-zinc-400 font-mono ml-8">
            {{ dataDir || 'Unknown' }}
          </p>
        </div>

        <div v-if="compatInfo">
          <div class="flex items-center gap-x-3">
            <CubeIcon class="h-5 w-5 text-zinc-400" />
            <h3 class="text-sm font-medium leading-6 text-zinc-100">Compatibility Settings</h3>
          </div>
          <div class="mt-2 ml-8 space-y-1">
            <p class="text-sm text-zinc-400 font-mono">
              Enabled: {{ compatInfo.enabled ? 'Yes' : 'No' }}
            </p>
            <p class="text-sm text-zinc-400 font-mono">
              Runner: {{ compatInfo.runner || 'Not configured' }}
            </p>
            <p class="text-sm text-zinc-400 font-mono">
              Prefix: {{ compatInfo.prefix || 'Not configured' }}
            </p>
          </div>
        </div>

        <div class="pt-6">
          <button
            @click="openLogFile"
            type="button"
            class="inline-flex items-center gap-x-2 rounded-md bg-blue-600 px-3.5 py-2.5 text-sm font-semibold text-white shadow-sm hover:bg-blue-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue-600"
          >
            <DocumentTextIcon class="h-5 w-5" aria-hidden="true" />
            Open Log File
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { platform, type } from '@tauri-apps/plugin-os';
import { 
  FingerPrintIcon, 
  TagIcon,
  ComputerDesktopIcon,
  ServerIcon,
  FolderIcon,
  CubeIcon,
  DocumentTextIcon,
} from "@heroicons/vue/24/outline";
import { open } from '@tauri-apps/plugin-shell';

const clientId = ref<string | null>(null);
const platformInfo = ref('Loading...');
const serverUrl = ref<string | null>(null);
const dataDir = ref<string | null>(null);
const compatInfo = ref<{
  enabled: boolean;
  runner: string | null;
  prefix: string | null;
} | null>(null);

onMounted(async () => {
  try {
    // Fetch client ID
    const id = await invoke<string | null>('fetch_client_id');
    clientId.value = id;

    // Get platform info
    const plat = await platform();
    const osType = await type();
    platformInfo.value = `${plat} (${osType})`;

    // Get base URL from database
    const baseUrl = await invoke<string | null>('fetch_base_url');
    serverUrl.value = baseUrl;

    // Get debug info (data dir and compat settings)
    const debugInfo = await invoke<{
      dataDir: string;
      compatibility: null | {
        enabled: boolean;
        runner: string | null;
        prefix: string | null;
      };
    }>('fetch_umu_info');
    
    dataDir.value = debugInfo.dataDir;
    compatInfo.value = debugInfo.compatibility;

  } catch (error) {
    console.error('Failed to fetch debug info:', error);
  }
});

async function openLogFile() {
  if (dataDir.value) {
    try {
      const logPath = `${dataDir.value}/drop.log`;
      await open(logPath);
    } catch (error) {
      console.error('Failed to open log file:', error)
    }
  }
}
</script> 
