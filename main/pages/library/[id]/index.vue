<template>
  <div
    class="mx-auto w-full relative flex flex-col justify-center pt-72 overflow-hidden"
  >
    <div class="absolute inset-0 z-0">
      <img
        :src="bannerUrl"
        class="w-full h-[24rem] object-cover blur-sm scale-105"
      />
      <div
        class="absolute inset-0 bg-gradient-to-t from-zinc-900 via-zinc-900/80 to-transparent opacity-90"
      />
      <div
        class="absolute inset-0 bg-gradient-to-r from-zinc-900/95 via-zinc-900/80 to-transparent opacity-90"
      />
    </div>

    <div class="relative z-10">
      <div class="px-8 pb-4">
        <h1
          class="text-5xl text-zinc-100 font-bold font-display drop-shadow-lg mb-8"
        >
          {{ game.mName }}
        </h1>

        <div class="flex flex-row gap-x-4 items-stretch mb-8">
          <!-- Do not add scale animations to this: https://stackoverflow.com/a/35683068 -->
          <GameStatusButton
            @install="() => installFlow()"
            @launch="() => launch()"
            @queue="() => queue()"
            @uninstall="() => uninstall()"
            @kill="() => kill()"
            @options="() => (configureModalOpen = true)"
            @resume="() => resumeDownload()"
            :status="status"
          />
          <a
            :href="remoteUrl"
            target="_blank"
            type="button"
            class="transition-transform duration-300 hover:scale-105 active:scale-95 inline-flex items-center rounded-md bg-zinc-800/50 px-6 font-semibold text-white shadow-xl backdrop-blur-sm hover:bg-zinc-800/80 uppercase font-display"
          >
            <BuildingStorefrontIcon class="mr-2 size-5" aria-hidden="true" />
            Store
          </a>
        </div>
      </div>

      <!-- Main content -->
      <div class="w-full bg-zinc-900 px-8 py-6">
        <div class="grid grid-cols-[2fr,1fr] gap-8">
          <div class="space-y-6">
            <div class="bg-zinc-800/50 rounded-xl p-6 backdrop-blur-sm">
              <div
                v-html="htmlDescription"
                class="prose prose-invert prose-blue overflow-y-auto custom-scrollbar max-w-none"
              ></div>
            </div>
          </div>

          <div class="space-y-6">
            <div class="bg-zinc-800/50 rounded-xl p-6 backdrop-blur-sm">
              <h2 class="text-xl font-display font-semibold text-zinc-100 mb-4">
                Game Images
              </h2>
              <div class="relative">
                <div v-if="mediaUrls.length > 0">
                  <div
                    class="relative aspect-video rounded-lg overflow-hidden cursor-pointer group"
                  >
                    <div
                      class="absolute inset-0"
                      @click="fullscreenImage = mediaUrls[currentImageIndex]"
                    >
                      <TransitionGroup name="slide" tag="div" class="h-full">
                        <img
                          v-for="(url, index) in mediaUrls"
                          :key="index"
                          :src="url"
                          class="absolute inset-0 w-full h-full object-cover"
                          v-show="index === currentImageIndex"
                        />
                      </TransitionGroup>
                    </div>

                    <div
                      class="absolute inset-0 flex items-center justify-between px-4 opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none"
                    >
                      <div class="pointer-events-auto">
                        <button
                          v-if="mediaUrls.length > 1"
                          @click.stop="previousImage()"
                          class="p-2 rounded-full bg-zinc-900/50 text-zinc-100 hover:bg-zinc-900/80 transition-all duration-300 hover:scale-110"
                        >
                          <ChevronLeftIcon class="size-5" />
                        </button>
                      </div>
                      <div class="pointer-events-auto">
                        <button
                          v-if="mediaUrls.length > 1"
                          @click.stop="nextImage()"
                          class="p-2 rounded-full bg-zinc-900/50 text-zinc-100 hover:bg-zinc-900/80 transition-all duration-300 hover:scale-110"
                        >
                          <ChevronRightIcon class="size-5" />
                        </button>
                      </div>
                    </div>

                    <div
                      class="absolute inset-0 bg-gradient-to-t from-black/50 to-transparent opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none"
                    />
                    <div
                      class="absolute bottom-4 right-4 flex items-center gap-x-2 text-white opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none"
                    >
                      <ArrowsPointingOutIcon class="size-5" />
                      <span class="text-sm font-medium">View Fullscreen</span>
                    </div>
                  </div>

                  <div
                    class="absolute -bottom-2 left-1/2 -translate-x-1/2 flex gap-x-2"
                  >
                    <button
                      v-for="(_, index) in mediaUrls"
                      :key="index"
                      @click.stop="currentImageIndex = index"
                      class="w-1.5 h-1.5 rounded-full transition-all"
                      :class="[
                        currentImageIndex === index
                          ? 'bg-zinc-100 scale-125'
                          : 'bg-zinc-600 hover:bg-zinc-500',
                      ]"
                    />
                  </div>
                </div>

                <div
                  v-else
                  class="aspect-video rounded-lg overflow-hidden bg-zinc-900/50 flex flex-col items-center justify-center text-center px-4"
                >
                  <PhotoIcon class="size-12 text-zinc-500 mb-2" />
                  <p class="text-zinc-400 font-medium">No images available</p>
                  <p class="text-zinc-500 text-sm">
                    Game screenshots will appear here when available
                  </p>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>

  <ModalTemplate v-model="installFlowOpen">
    <template #default>
      <div class="sm:flex sm:items-start">
        <div class="mt-3 text-center sm:mt-0 sm:text-left">
          <h3 class="text-base font-semibold text-zinc-100">
            Install {{ game.mName }}?
          </h3>
          <div class="mt-2">
            <p class="text-sm text-zinc-400">
              Drop will add {{ game.mName }} to the queue to be downloaded.
              While downloading, Drop may use up a large amount of resources,
              particularly network bandwidth and CPU utilisation.
            </p>
          </div>
        </div>
      </div>

      <form class="space-y-6">
        <div v-if="versionOptions && versionOptions.length > 0">
          <Listbox as="div" v-model="installVersionIndex">
            <ListboxLabel class="block text-sm/6 font-medium text-zinc-100"
              >Version</ListboxLabel
            >
            <div class="relative mt-2">
              <ListboxButton
                class="relative w-full cursor-default rounded-md bg-zinc-800 py-1.5 pl-3 pr-10 text-left text-zinc-100 shadow-sm ring-1 ring-inset ring-zinc-700 focus:outline-none focus:ring-2 focus:ring-blue-600 sm:text-sm/6"
              >
                <span class="block truncate"
                  >{{ versionOptions[installVersionIndex].versionName }}
                  on
                  {{ versionOptions[installVersionIndex].platform }}</span
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
                        active ? 'bg-blue-600 text-white' : 'text-zinc-300',
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
                        <CheckIcon class="h-5 w-5" aria-hidden="true" />
                      </span>
                    </li>
                  </ListboxOption>
                </ListboxOptions>
              </transition>
            </div>
          </Listbox>
        </div>
        <div
          v-else-if="versionOptions === null || versionOptions?.length == 0"
          class="mt-1 rounded-md bg-red-600/10 p-4"
        >
          <div class="flex">
            <div class="flex-shrink-0">
              <XCircleIcon class="h-5 w-5 text-red-600" aria-hidden="true" />
            </div>
            <div class="ml-3">
              <h3 class="text-sm font-medium text-red-600">
                There are no supported versions to install. Please contact your
                server admin or try again later.
              </h3>
            </div>
          </div>
        </div>
        <div v-else class="w-full flex items-center justify-center p-4">
          <div role="status">
            <svg
              aria-hidden="true"
              class="w-7 h-7 text-transparent animate-spin fill-white"
              viewBox="0 0 100 101"
              fill="none"
              xmlns="http://www.w3.org/2000/svg"
            >
              <path
                d="M100 50.5908C100 78.2051 77.6142 100.591 50 100.591C22.3858 100.591 0 78.2051 0 50.5908C0 22.9766 22.3858 0.59082 50 0.59082C77.6142 0.59082 100 22.9766 100 50.5908ZM9.08144 50.5908C9.08144 73.1895 27.4013 91.5094 50 91.5094C72.5987 91.5094 90.9186 73.1895 90.9186 50.5908C90.9186 27.9921 72.5987 9.67226 50 9.67226C27.4013 9.67226 9.08144 27.9921 9.08144 50.5908Z"
                fill="currentColor"
              />
              <path
                d="M93.9676 39.0409C96.393 38.4038 97.8624 35.9116 97.0079 33.5539C95.2932 28.8227 92.871 24.3692 89.8167 20.348C85.8452 15.1192 80.8826 10.7238 75.2124 7.41289C69.5422 4.10194 63.2754 1.94025 56.7698 1.05124C51.7666 0.367541 46.6976 0.446843 41.7345 1.27873C39.2613 1.69328 37.813 4.19778 38.4501 6.62326C39.0873 9.04874 41.5694 10.4717 44.0505 10.1071C47.8511 9.54855 51.7191 9.52689 55.5402 10.0491C60.8642 10.7766 65.9928 12.5457 70.6331 15.2552C75.2735 17.9648 79.3347 21.5619 82.5849 25.841C84.9175 28.9121 86.7997 32.2913 88.1811 35.8758C89.083 38.2158 91.5421 39.6781 93.9676 39.0409Z"
                fill="currentFill"
              />
            </svg>
            <span class="sr-only">Loading...</span>
          </div>
        </div>
        <div v-if="installDirs">
          <Listbox as="div" v-model="installDir">
            <ListboxLabel class="block text-sm/6 font-medium text-zinc-100"
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
                        active ? 'bg-blue-600 text-white' : 'text-zinc-300',
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
                        >{{ dir }}</span
                      >

                      <span
                        v-if="selected"
                        :class="[
                          active ? 'text-white' : 'text-blue-600',
                          'absolute inset-y-0 right-0 flex items-center pr-4',
                        ]"
                      >
                        <CheckIcon class="h-5 w-5" aria-hidden="true" />
                      </span>
                    </li>
                  </ListboxOption>
                </ListboxOptions>
              </transition>
            </div>
            <div class="text-zinc-400 text-sm mt-2">
              Add more install directories in
              <PageWidget to="/settings/downloads">
                <WrenchIcon class="size-3" />
                Settings
              </PageWidget>
            </div>
          </Listbox>
        </div>
      </form>

      <div v-if="installError" class="mt-1 rounded-md bg-red-600/10 p-4">
        <div class="flex">
          <div class="flex-shrink-0">
            <XCircleIcon class="h-5 w-5 text-red-600" aria-hidden="true" />
          </div>
          <div class="ml-3">
            <h3 class="text-sm font-medium text-red-600">
              {{ installError }}
            </h3>
          </div>
        </div>
      </div>
    </template>
    <template #buttons>
      <LoadingButton
        @click="() => install()"
        :disabled="!(versionOptions && versionOptions.length > 0)"
        :loading="installLoading"
        type="submit"
        class="ml-2 w-full sm:w-fit"
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
    </template>
  </ModalTemplate>

  <!-- 
  Dear future DecDuck,
  This v-if is necessary for Vue rendering reasons
  (it tries to access the game version for not installed games)
  You have already tried to remove it
  Don't.  
  -->
  <GameOptionsModal
    v-if="status.type === GameStatusEnum.Installed"
    v-model="configureModalOpen"
    :game-id="game.id"
  />

  <Transition
    enter="transition ease-out duration-300"
    enter-from="opacity-0"
    enter-to="opacity-100"
    leave="transition ease-in duration-200"
    leave-from="opacity-100"
    leave-to="opacity-0"
  >
    <div
      v-if="fullscreenImage"
      class="fixed inset-0 z-50 bg-black/95 flex items-center justify-center"
      @click="fullscreenImage = null"
    >
      <div
        class="relative w-full h-full flex items-center justify-center"
        @click.stop
      >
        <button
          class="absolute top-4 right-4 p-2 rounded-full bg-zinc-900/50 text-zinc-100 hover:bg-zinc-900 transition-colors"
          @click.stop="fullscreenImage = null"
        >
          <XMarkIcon class="size-6" />
        </button>

        <button
          v-if="mediaUrls.length > 1"
          @click.stop="previousImage()"
          class="absolute left-4 p-3 rounded-full bg-zinc-900/50 text-zinc-100 hover:bg-zinc-900 transition-colors"
        >
          <ChevronLeftIcon class="size-6" />
        </button>
        <button
          v-if="mediaUrls.length > 1"
          @click.stop="nextImage()"
          class="absolute right-4 p-3 rounded-full bg-zinc-900/50 text-zinc-100 hover:bg-zinc-900 transition-colors"
        >
          <ChevronRightIcon class="size-6" />
        </button>

        <TransitionGroup
          name="slide"
          tag="div"
          class="w-full h-full flex items-center justify-center"
          @click.stop
        >
          <img
            v-for="(url, index) in mediaUrls"
            v-show="currentImageIndex === index"
            :key="index"
            :src="url"
            class="max-h-[90vh] max-w-[90vw] object-contain"
            :alt="`${game.mName} screenshot ${index + 1}`"
          />
        </TransitionGroup>

        <div
          class="absolute bottom-4 left-1/2 -translate-x-1/2 px-4 py-2 rounded-full bg-zinc-900/50 backdrop-blur-sm"
        >
          <p class="text-zinc-100 text-sm font-medium">
            {{ currentImageIndex + 1 }} / {{ mediaUrls.length }}
          </p>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import {
  Listbox,
  ListboxButton,
  ListboxLabel,
  ListboxOption,
  ListboxOptions,
} from "@headlessui/vue";
import {
  CheckIcon,
  ChevronUpDownIcon,
  WrenchIcon,
  ChevronLeftIcon,
  ChevronRightIcon,
  XMarkIcon,
  ArrowsPointingOutIcon,
  PhotoIcon,
} from "@heroicons/vue/20/solid";
import { BuildingStorefrontIcon } from "@heroicons/vue/24/outline";
import { XCircleIcon } from "@heroicons/vue/24/solid";
import { invoke } from "@tauri-apps/api/core";
import { micromark } from "micromark";
import { GameStatusEnum } from "~/types";

const route = useRoute();
const router = useRouter();
const id = route.params.id.toString();

const { game: rawGame, status } = await useGame(id);
const game = ref(rawGame);

const remoteUrl: string = await invoke("gen_drop_url", {
  path: `/store/${game.value.id}`,
});

const bannerUrl = await useObject(game.value.mBannerObjectId);

// Get all available images
const mediaUrls = await Promise.all(
  game.value.mImageCarouselObjectIds.map(async (v) => {
    const src = await useObject(v);
    return src;
  })
);

const htmlDescription = micromark(game.value.mDescription);

const installFlowOpen = ref(false);
const versionOptions = ref<
  undefined | Array<{ versionName: string; platform: string }>
>();
const installDirs = ref<undefined | Array<string>>();
const currentImageIndex = ref(0);

const configureModalOpen = ref(false);

async function installFlow() {
  installFlowOpen.value = true;
  versionOptions.value = undefined;
  installDirs.value = undefined;

  try {
    versionOptions.value = await invoke("fetch_game_version_options", {
      gameId: game.value.id,
    });
    installDirs.value = await invoke("fetch_download_dir_stats");
  } catch (error) {
    installError.value = (error as string).toString();
    versionOptions.value = undefined;
  }
}

const installLoading = ref(false);
const installError = ref<string | undefined>();
const installVersionIndex = ref(0);
const installDir = ref(0);
async function install() {
  try {
    if (!versionOptions.value) throw new Error("Versions have not been loaded");
    installLoading.value = true;
    await invoke("download_game", {
      gameId: game.value.id,
      gameVersion: versionOptions.value[installVersionIndex.value].versionName,
      installDir: installDir.value,
    });
    installFlowOpen.value = false;
  } catch (error) {
    installError.value = (error as string).toString();
  }

  installLoading.value = false;
}

async function resumeDownload() {
  try {
    await invoke("resume_download", { gameId: game.value.id });
  } catch (e) {
    console.error(e);
  }
}

async function launch() {
  try {
    await invoke("launch_game", { id: game.value.id });
  } catch (e) {
    createModal(
      ModalType.Notification,
      {
        title: `Couldn't run "${game.value.mName}"`,
        description: `Drop failed to launch "${game.value.mName}": ${e}`,
        buttonText: "Close",
      },
      (e, c) => c()
    );
    console.error(e);
  }
}

async function queue() {
  router.push("/queue");
}

async function uninstall() {
  await invoke("uninstall_game", { gameId: game.value.id });
}

async function kill() {
  try {
    await invoke("kill_game", { gameId: game.value.id });
  } catch (e) {
    createModal(
      ModalType.Notification,
      {
        title: `Couldn't stop "${game.value.mName}"`,
        description: `Drop failed to stop "${game.value.mName}": ${e}`,
        buttonText: "Close",
      },
      (e, c) => c()
    );
    console.error(e);
  }
}

function nextImage() {
  currentImageIndex.value = (currentImageIndex.value + 1) % mediaUrls.length;
}

function previousImage() {
  currentImageIndex.value =
    (currentImageIndex.value - 1 + mediaUrls.length) % mediaUrls.length;
}

const fullscreenImage = ref<string | null>(null);
</script>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.slide-enter-active,
.slide-leave-active {
  transition: all 0.3s ease;
  position: absolute;
}

.slide-enter-from {
  opacity: 0;
  transform: translateX(100%);
}

.slide-leave-to {
  opacity: 0;
  transform: translateX(-100%);
}

.custom-scrollbar {
  scrollbar-width: thin;
  scrollbar-color: rgb(82 82 91) transparent;
}

.custom-scrollbar::-webkit-scrollbar {
  width: 6px;
}

.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background-color: rgb(82 82 91);
  border-radius: 3px;
}
</style>
