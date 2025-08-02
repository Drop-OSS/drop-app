<template>
  <div class="p-8">
    <!-- Page Header -->
    <div class="mb-8">
      <h2 class="text-3xl font-bold font-display text-zinc-100 mb-2">
        Settings
      </h2>
      <p class="text-lg text-zinc-400">
        Configure your Big Picture experience
      </p>
    </div>

    <!-- Settings Sections -->
    <div class="space-y-8">
      <!-- General Settings -->
      <div class="bg-zinc-800/50 rounded-xl p-6 backdrop-blur-sm">
        <h3 class="text-xl font-semibold text-zinc-100 mb-6 flex items-center">
          <Cog6ToothIcon class="h-6 w-6 mr-3 text-blue-400" />
          General Settings
        </h3>
        
        <div class="space-y-6">
          <!-- Autostart Setting -->
          <div class="flex items-center justify-between p-4 bg-zinc-900/50 rounded-lg">
            <div class="flex-1">
              <h4 class="text-lg font-medium text-zinc-100 mb-1">
                Start with system
              </h4>
              <p class="text-sm text-zinc-400">
                Drop will automatically start when you log into your computer
              </p>
            </div>
            <Switch
              v-model="autostartEnabled"
              :class="[
                autostartEnabled ? 'bg-blue-600' : 'bg-zinc-700',
                'relative inline-flex h-8 w-14 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out',
              ]"
            >
              <span
                :class="[
                  autostartEnabled ? 'translate-x-6' : 'translate-x-0',
                  'pointer-events-none relative inline-block h-7 w-7 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out',
                ]"
              />
            </Switch>
          </div>

          <!-- Start in Big Picture Mode Setting -->
          <div class="flex items-center justify-between p-4 bg-zinc-900/50 rounded-lg">
            <div class="flex-1">
              <h4 class="text-lg font-medium text-zinc-100 mb-1">
                Start Drop in Big Picture Mode
              </h4>
              <p class="text-sm text-zinc-400">
                Drop will automatically start in Big Picture mode when launched
              </p>
            </div>
            <Switch
              v-model="startInBigPicture"
              :class="[
                startInBigPicture ? 'bg-blue-600' : 'bg-zinc-700',
                'relative inline-flex h-8 w-14 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out',
              ]"
            >
              <span
                :class="[
                  startInBigPicture ? 'translate-x-6' : 'translate-x-0',
                  'pointer-events-none relative inline-block h-7 w-7 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out',
                ]"
              />
            </Switch>
          </div>

          <!-- Fullscreen Setting -->
          <div class="flex items-center justify-between p-4 bg-zinc-900/50 rounded-lg">
            <div class="flex-1">
              <h4 class="text-lg font-medium text-zinc-100 mb-1">
                Always fullscreen in Big Picture
              </h4>
              <p class="text-sm text-zinc-400">
                Automatically enter fullscreen when switching to Big Picture mode
              </p>
            </div>
            <Switch
              v-model="autoFullscreen"
              :class="[
                autoFullscreen ? 'bg-blue-600' : 'bg-zinc-700',
                'relative inline-flex h-8 w-14 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out',
              ]"
            >
              <span
                :class="[
                  autoFullscreen ? 'translate-x-6' : 'translate-x-0',
                  'pointer-events-none relative inline-block h-7 w-7 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out',
                ]"
              />
            </Switch>
          </div>
        </div>
      </div>



             <!-- Downloads Settings -->
       <div class="bg-zinc-800/50 rounded-xl p-6 backdrop-blur-sm">
         <h3 class="text-xl font-semibold text-zinc-100 mb-6 flex items-center">
           <ArrowDownTrayIcon class="h-6 w-6 mr-3 text-green-400" />
           Downloads
         </h3>
         
         <div class="space-y-6">
           <!-- Download Threads -->
           <div class="p-4 bg-zinc-900/50 rounded-lg">
             <h4 class="text-lg font-medium text-zinc-100 mb-3">
               Maximum Download Threads
             </h4>
             <input 
               type="number" 
               min="1" 
               max="32" 
               v-model="downloadThreads"
               class="w-full px-4 py-3 bg-zinc-800 border border-zinc-700 rounded-lg text-zinc-100 focus:outline-none focus:ring-2 focus:ring-blue-600 focus:border-transparent"
             />
             <p class="text-sm text-zinc-400 mt-2">
               Higher values may download faster but use more system resources
             </p>
           </div>

           <!-- Force Offline -->
           <div class="flex items-center justify-between p-4 bg-zinc-900/50 rounded-lg">
             <div class="flex-1">
               <h4 class="text-lg font-medium text-zinc-100 mb-1">
                 Force Offline Mode
               </h4>
               <p class="text-sm text-zinc-400">
                 Drop will not make any external connections
               </p>
             </div>
             <Switch
               v-model="forceOffline"
               :class="[
                 forceOffline ? 'bg-blue-600' : 'bg-zinc-700',
                 'relative inline-flex h-8 w-14 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out',
               ]"
             >
               <span
                 :class="[
                   forceOffline ? 'translate-x-6' : 'translate-x-0',
                   'pointer-events-none relative inline-block h-7 w-7 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out',
                 ]"
               />
             </Switch>
           </div>
         </div>
       </div>

       <!-- Account Settings -->
       <div class="bg-zinc-800/50 rounded-xl p-6 backdrop-blur-sm">
         <h3 class="text-xl font-semibold text-zinc-100 mb-6 flex items-center">
           <UserIcon class="h-6 w-6 mr-3 text-purple-400" />
           Account
         </h3>
         
         <div class="space-y-6">
           <div class="flex items-center justify-between p-4 bg-zinc-900/50 rounded-lg">
             <div class="flex-1">
               <h4 class="text-lg font-medium text-zinc-100 mb-1">
                 Sign Out
               </h4>
               <p class="text-sm text-zinc-400">
                 Sign out of your Drop account on this device
               </p>
             </div>
             <button
               @click="signOut"
               class="px-6 py-3 bg-red-600 hover:bg-red-500 text-white font-semibold rounded-lg transition-colors duration-200"
             >
               Sign Out
             </button>
           </div>
           
           <div v-if="signOutError" class="p-4 bg-red-600/10 rounded-lg">
             <div class="flex items-center">
               <XCircleIcon class="h-5 w-5 text-red-600 mr-3" />
               <span class="text-red-600">{{ signOutError }}</span>
             </div>
           </div>
         </div>
       </div>

       <!-- Debug Information -->
       <div class="bg-zinc-800/50 rounded-xl p-6 backdrop-blur-sm">
         <h3 class="text-xl font-semibold text-zinc-100 mb-6 flex items-center">
           <BugAntIcon class="h-6 w-6 mr-3 text-orange-400" />
           Debug Information
         </h3>
         
         <div class="space-y-4">
           <div class="flex items-center justify-between p-4 bg-zinc-900/50 rounded-lg">
             <span class="text-zinc-100">Client ID</span>
             <span class="text-zinc-400 font-mono text-sm">{{ clientId || "Not signed in" }}</span>
           </div>
           <div class="flex items-center justify-between p-4 bg-zinc-900/50 rounded-lg">
             <span class="text-zinc-100">Platform</span>
             <span class="text-zinc-400 font-mono text-sm">{{ platformInfo }}</span>
           </div>
           <div class="flex items-center justify-between p-4 bg-zinc-900/50 rounded-lg">
             <span class="text-zinc-100">Server URL</span>
             <span class="text-zinc-400 font-mono text-sm">{{ baseUrl || "Not connected" }}</span>
           </div>
           <div class="flex items-center justify-between p-4 bg-zinc-900/50 rounded-lg">
             <span class="text-zinc-100">Data Directory</span>
             <span class="text-zinc-400 font-mono text-sm">{{ dataDir || "Unknown" }}</span>
           </div>
           
           <div class="flex gap-4 pt-4">
             <button
               @click="openDataDir"
               class="flex-1 px-4 py-3 bg-blue-600 hover:bg-blue-500 text-white font-semibold rounded-lg transition-colors duration-200 flex items-center justify-center"
             >
               <FolderIcon class="h-5 w-5 mr-2" />
               Open Data Directory
             </button>
             <button
               @click="openLogFile"
               class="flex-1 px-4 py-3 bg-blue-600 hover:bg-blue-500 text-white font-semibold rounded-lg transition-colors duration-200 flex items-center justify-center"
             >
               <DocumentTextIcon class="h-5 w-5 mr-2" />
               Open Log File
             </button>
           </div>
         </div>
       </div>

       <!-- About Section -->
       <div class="bg-zinc-800/50 rounded-xl p-6 backdrop-blur-sm">
         <h3 class="text-xl font-semibold text-zinc-100 mb-6 flex items-center">
           <InformationCircleIcon class="h-6 w-6 mr-3 text-yellow-400" />
           About
         </h3>
         
         <div class="space-y-4">
           <div class="flex items-center justify-between p-4 bg-zinc-900/50 rounded-lg">
             <span class="text-zinc-100">Version</span>
             <span class="text-zinc-400">1.0.0</span>
           </div>
           <div class="flex items-center justify-between p-4 bg-zinc-900/50 rounded-lg">
             <span class="text-zinc-100">Build Date</span>
             <span class="text-zinc-400">{{ new Date().toLocaleDateString() }}</span>
           </div>
           <div class="flex items-center justify-between p-4 bg-zinc-900/50 rounded-lg">
             <span class="text-zinc-100">Platform</span>
             <span class="text-zinc-400">Desktop</span>
           </div>
         </div>
       </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Switch } from "@headlessui/vue";
import { 
  Cog6ToothIcon, 
  InformationCircleIcon,
  ArrowDownTrayIcon,
  UserIcon,
  BugAntIcon,
  XCircleIcon,
  FolderIcon,
  DocumentTextIcon
} from "@heroicons/vue/24/outline";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useRouter } from "#imports";
import { platform } from '@tauri-apps/plugin-os';
import { open } from "@tauri-apps/plugin-shell";
import { type Settings } from "~/types";

definePageMeta({
  layout: "big-picture"
});

const router = useRouter();

// Settings state
const autostartEnabled = ref<boolean>(false);
const autoFullscreen = ref<boolean>(true);
const startInBigPicture = ref<boolean>(false);

// Download settings
const settings = await invoke<Settings>("fetch_settings");
const downloadThreads = ref(settings?.maxDownloadThreads ?? 4);
const forceOffline = ref(settings?.forceOffline ?? false);

// Account settings
const signOutError = ref<string | null>(null);

// Debug information
const clientId = ref<string | null>(null);
const platformInfo = ref("Loading...");
const baseUrl = ref<string | null>(null);
const dataDir = ref<string | null>(null);

// Load system data
const systemData = await invoke<{
  clientId: string;
  baseUrl: string;
  dataDir: string;
}>("fetch_system_data");

clientId.value = systemData.clientId;
baseUrl.value = systemData.baseUrl;
dataDir.value = systemData.dataDir;

// Load platform info
const currentPlatform = await platform();
platformInfo.value = currentPlatform;

// Load initial state
onMounted(async () => {
  try {
    const enabled = await invoke("get_autostart_enabled");
    autostartEnabled.value = enabled as boolean;
  } catch (error) {
    console.error("Failed to load autostart setting:", error);
  }

  try {
    const bigPictureEnabled = await invoke("get_start_in_big_picture");
    startInBigPicture.value = bigPictureEnabled as boolean;
  } catch (error) {
    console.error("Failed to load big picture setting:", error);
  }

  // Listen for auth events
  await listen("auth/signedout", () => {
    router.push("/auth/signedout");
  });
});

// Watch for autostart changes
watch(autostartEnabled, async (newValue: boolean) => {
  try {
    await invoke("toggle_autostart", { enabled: newValue });
  } catch (error) {
    console.error("Failed to toggle autostart:", error);
    autostartEnabled.value = !newValue;
  }
});

// Watch for big picture mode changes
watch(startInBigPicture, async (newValue: boolean) => {
  try {
    await invoke("set_start_in_big_picture", { enabled: newValue });
  } catch (error) {
    console.error("Failed to set big picture setting:", error);
    startInBigPicture.value = !newValue;
  }
});

// Watch for auto fullscreen changes
watch(autoFullscreen, (newValue: boolean) => {
  // Save to localStorage or settings
  localStorage.setItem("bigPictureAutoFullscreen", newValue.toString());
});

// Load auto fullscreen setting
onMounted(() => {
  const saved = localStorage.getItem("bigPictureAutoFullscreen");
  if (saved !== null) {
    autoFullscreen.value = saved === "true";
  }
});

// Sign out function
async function signOut() {
  try {
    signOutError.value = null;
    await invoke("sign_out");
  } catch (e) {
    signOutError.value = `Failed to sign out: ${e}`;
  }
}

// Debug functions
async function openDataDir() {
  if (!dataDir.value) return;
  try {
    await open(dataDir.value);
  } catch (error) {
    console.error("Failed to open data dir:", error);
  }
}

async function openLogFile() {
  if (!dataDir.value) return;
  try {
    const logPath = `${dataDir.value}/drop.log`;
    await open(logPath);
  } catch (error) {
    console.error("Failed to open log file:", error);
  }
}
</script>

 