<template>
  <div
    class="mx-auto w-full relative flex flex-col justify-center pt-64 z-10 overflow-hidden"
  >
    <!-- banner image -->
    <div class="absolute flex top-0 h-fit inset-x-0 z-[-20]">
      <img :src="bannerUrl" class="w-full h-auto object-cover" />
      <h1
        class="absolute inset-x-0 w-full text-center top-32 -translate-y-[50%] text-4xl text-zinc-100 font-bold font-display z-50"
      >
        {{ game.mName }}
      </h1>
      <div
        class="absolute inset-0 bg-gradient-to-b from-transparent to-50% to-zinc-900"
      />
    </div>
    <!-- main page -->
    <div class="w-full min-h-screen mx-auto bg-zinc-900 px-5 py-6">
      <!-- game toolbar -->
      <div>
        <GameStatusButton @install="() => installFlow()" :status="status" />
      </div>
    </div>
  </div>

  <TransitionRoot as="template" :show="installFlowOpen">
    <Dialog class="relative z-50" @close="installFlowOpen = false">
      <TransitionChild
        as="template"
        enter="ease-out duration-300"
        enter-from="opacity-0"
        enter-to="opacity-100"
        leave="ease-in duration-200"
        leave-from="opacity-100"
        leave-to="opacity-0"
      >
        <div
          class="fixed inset-0 bg-zinc-950 bg-opacity-75 transition-opacity"
        />
      </TransitionChild>

      <div class="fixed inset-0 z-10 w-screen overflow-y-auto">
        <div
          class="flex min-h-full items-start justify-center p-4 text-center sm:items-center sm:p-0"
        >
          <TransitionChild
            as="template"
            enter="ease-out duration-300"
            enter-from="opacity-0 translate-y-4 sm:translate-y-0 sm:scale-95"
            enter-to="opacity-100 translate-y-0 sm:scale-100"
            leave="ease-in duration-200"
            leave-from="opacity-100 translate-y-0 sm:scale-100"
            leave-to="opacity-0 translate-y-4 sm:translate-y-0 sm:scale-95"
          >
            <form
              @submit.prevent="() => install()"
              class="relative transform rounded-lg bg-zinc-900 text-left shadow-xl transition-all sm:my-8 sm:w-full sm:max-w-lg"
            >
              <div class="px-4 pb-4 pt-5 space-y-4 sm:p-6 sm:pb-4">
                <div class="sm:flex sm:items-start">
                  <div class="mt-3 text-center sm:mt-0 sm:text-left">
                    <DialogTitle
                      as="h3"
                      class="text-base font-semibold text-zinc-100"
                      >Install {{ game.mName }}?
                    </DialogTitle>
                    <div class="mt-2">
                      <p class="text-sm text-zinc-400">
                        Drop will add {{ game.mName }} to the queue to be
                        downloaded. While downloading, Drop may use up a large
                        amount of resources, particularly network bandwidth and
                        CPU utilisation.
                      </p>
                    </div>
                  </div>
                </div>

                <div class="space-y-6">
                  <div v-if="versionOptions && versionOptions.length > 0">
                    <Listbox as="div" v-model="installVersionIndex">
                      <ListboxLabel
                        class="block text-sm/6 font-medium text-zinc-100"
                        >Version</ListboxLabel
                      >
                      <div class="relative mt-2">
                        <ListboxButton
                          class="relative w-full cursor-default rounded-md bg-zinc-800 py-1.5 pl-3 pr-10 text-left text-zinc-100 shadow-sm ring-1 ring-inset ring-zinc-700 focus:outline-none focus:ring-2 focus:ring-blue-600 sm:text-sm/6"
                        >
                          <span class="block truncate"
                            >{{
                              versionOptions[installVersionIndex].versionName
                            }}
                            on
                            {{
                              versionOptions[installVersionIndex].platform
                            }}</span
                          >
                          <span
                            class="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-2"
                          >
                            <ChevronUpDownIcon
                              class="h-5 w-5 text-gray-400"
                              aria-hidden="true"
                            />
                          </span>
                        </ListboxButton>

                        <transition
                          leave-active-class="transition ease-in duration-100"
                          leave-from-class="opacity-100"
                          leave-to-class="opacity-0"
                        >
                          <ListboxOptions
                            class="absolute z-10 mt-1 max-h-60 w-full overflow-auto rounded-md bg-zinc-900 py-1 text-base shadow-lg ring-1 ring-black ring-opacity-5 focus:outline-none sm:text-sm"
                          >
                            <ListboxOption
                              as="template"
                              v-for="(version, versionIdx) in versionOptions"
                              :key="version.versionName"
                              :value="versionIdx"
                              v-slot="{ active, selected }"
                            >
                              <li
                                :class="[
                                  active
                                    ? 'bg-blue-600 text-white'
                                    : 'text-zinc-300',
                                  'relative cursor-default select-none py-2 pl-3 pr-9',
                                ]"
                              >
                                <span
                                  :class="[
                                    selected
                                      ? 'font-semibold text-zinc-100'
                                      : 'font-normal',
                                    'block truncate',
                                  ]"
                                  >{{ version.versionName }} on
                                  {{ version.platform }}</span
                                >

                                <span
                                  v-if="selected"
                                  :class="[
                                    active ? 'text-white' : 'text-blue-600',
                                    'absolute inset-y-0 right-0 flex items-center pr-4',
                                  ]"
                                >
                                  <CheckIcon
                                    class="h-5 w-5"
                                    aria-hidden="true"
                                  />
                                </span>
                              </li>
                            </ListboxOption>
                          </ListboxOptions>
                        </transition>
                      </div>
                    </Listbox>
                  </div>
                  <div v-else class="mt-1 rounded-md bg-red-600/10 p-4">
                    <div class="flex">
                      <div class="flex-shrink-0">
                        <XCircleIcon
                          class="h-5 w-5 text-red-600"
                          aria-hidden="true"
                        />
                      </div>
                      <div class="ml-3">
                        <h3 class="text-sm font-medium text-red-600">
                          There are no versions to install. Please contact your
                          server admin or try again later.
                        </h3>
                      </div>
                    </div>
                  </div>
                  <div v-if="installDirs">
                    <Listbox as="div" v-model="installDir">
                      <ListboxLabel
                        class="block text-sm/6 font-medium text-zinc-100"
                        >Install to</ListboxLabel
                      >
                      <div class="relative mt-2">
                        <ListboxButton
                          class="relative w-full cursor-default rounded-md bg-zinc-800 py-1.5 pl-3 pr-10 text-left text-zinc-100 shadow-sm ring-1 ring-inset ring-zinc-700 focus:outline-none focus:ring-2 focus:ring-blue-600 sm:text-sm/6"
                        >
                          <span class="block truncate">{{
                            installDirs[installDir]
                          }}</span>
                          <span
                            class="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-2"
                          >
                            <ChevronUpDownIcon
                              class="h-5 w-5 text-gray-400"
                              aria-hidden="true"
                            />
                          </span>
                        </ListboxButton>

                        <transition
                          leave-active-class="transition ease-in duration-100"
                          leave-from-class="opacity-100"
                          leave-to-class="opacity-0"
                        >
                          <ListboxOptions
                            class="absolute z-10 mt-1 max-h-60 w-full overflow-auto rounded-md bg-zinc-900 py-1 text-base shadow-lg ring-1 ring-black ring-opacity-5 focus:outline-none sm:text-sm"
                          >
                            <ListboxOption
                              as="template"
                              v-for="(dir, dirIdx) in installDirs"
                              :key="dir"
                              :value="dirIdx"
                              v-slot="{ active, selected }"
                            >
                              <li
                                :class="[
                                  active
                                    ? 'bg-blue-600 text-white'
                                    : 'text-zinc-300',
                                  'relative cursor-default select-none py-2 pl-3 pr-9',
                                ]"
                              >
                                <span
                                  :class="[
                                    selected
                                      ? 'font-semibold text-zinc-100'
                                      : 'font-normal',
                                    'block truncate',
                                  ]"
                                  >{{ dir }}}</span
                                >

                                <span
                                  v-if="selected"
                                  :class="[
                                    active ? 'text-white' : 'text-blue-600',
                                    'absolute inset-y-0 right-0 flex items-center pr-4',
                                  ]"
                                >
                                  <CheckIcon
                                    class="h-5 w-5"
                                    aria-hidden="true"
                                  />
                                </span>
                              </li>
                            </ListboxOption>
                          </ListboxOptions>
                        </transition>
                      </div>
                    </Listbox>
                  </div>
                </div>

                <div
                  v-if="installError"
                  class="mt-1 rounded-md bg-red-600/10 p-4"
                >
                  <div class="flex">
                    <div class="flex-shrink-0">
                      <XCircleIcon
                        class="h-5 w-5 text-red-600"
                        aria-hidden="true"
                      />
                    </div>
                    <div class="ml-3">
                      <h3 class="text-sm font-medium text-red-600">
                        {{ installError }}
                      </h3>
                    </div>
                  </div>
                </div>
              </div>
              <div
                class="rounded-b-lg bg-zinc-800 px-4 py-3 sm:flex sm:gap-x-2 sm:flex-row-reverse sm:px-6"
              >
                <LoadingButton
                  :disabled="
                    !(versionOptions && versionOptions.length > 0 && !installDir)
                  "
                  :loading="installLoading"
                  type="submit"
                  class="w-full sm:w-fit"
                >
                  Install
                </LoadingButton>
                <button
                  type="button"
                  class="mt-3 inline-flex w-full justify-center rounded-md bg-zinc-800 px-3 py-2 text-sm font-semibold text-zinc-100 shadow-sm ring-1 ring-inset ring-zinc-700 hover:bg-zinc-900 sm:mt-0 sm:w-auto"
                  @click="installFlowOpen = false"
                  ref="cancelButtonRef"
                >
                  Cancel
                </button>
              </div>
            </form>
          </TransitionChild>
        </div>
      </div>
    </Dialog>
  </TransitionRoot>
</template>

<script setup lang="ts">
import {
  Dialog,
  DialogTitle,
  TransitionChild,
  TransitionRoot,
  Listbox,
  ListboxButton,
  ListboxLabel,
  ListboxOption,
  ListboxOptions,
} from "@headlessui/vue";
import { CheckIcon, ChevronUpDownIcon } from "@heroicons/vue/20/solid";
import { XCircleIcon } from "@heroicons/vue/24/solid";

import type { Game } from "@prisma/client";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { GameStatus } from "~/types";

const route = useRoute();
const id = route.params.id;

const raw: { game: Game; status: GameStatus } = JSON.parse(
  await invoke<string>("fetch_game", { id: id })
);
const game = ref(raw.game);
const status = ref(raw.status);

listen(`update_game/${game.value.id}`, (event) => {
  const payload: { status: GameStatus } = event.payload as any;
  status.value = payload.status;
});

const bannerUrl = await useObject(game.value.mBannerId);

const installFlowOpen = ref(false);
const versionOptions = ref<
  undefined | Array<{ versionName: string; platform: string }>
>();
const installDirs = ref<undefined | Array<string>>();
async function installFlow() {
  installFlowOpen.value = true;

  try {
    versionOptions.value = await invoke("fetch_game_verion_options", {
      gameId: game.value.id,
    });
    installDirs.value = await invoke("fetch_download_dir_stats");
  } catch (error) {
    installError.value = (error as string).toString();
  }
}

const installLoading = ref(false);
const installError = ref<string | undefined>();
const installVersionIndex = ref(0);
const installDir = ref(0);
async function install() {
  try {
    if (!versionOptions.value)
      throw new Error("Versions have not been loaded.");
    installLoading.value = true;
    await invoke("download_game", {
      gameId: game.value.id,
      gameVersion: versionOptions.value[installVersionIndex.value].versionName,
      installDir: installDir.value,
    });
    installLoading.value = false;

    installFlowOpen.value = false;
  } catch (error) {
    installError.value = (error as string).toString();
  }
}
</script>
