<template>
  <div>
    <label for="launch" class="block text-sm/6 font-medium text-zinc-100"
      >Launch string template</label
    >
    <div class="mt-2">
      <input
        type="text"
        name="launch"
        id="launch"
        class="block w-full rounded-md bg-zinc-800 px-3 py-1.5 text-base text-zinc-100 outline-1 -outline-offset-1 outline-zinc-800 placeholder:text-zinc-400 focus:outline-2 focus:-outline-offset-2 focus:outline-blue-600 sm:text-sm/6"
        placeholder="{}"
        aria-describedby="launch-description"
        v-model="model.launchTemplate"
      />
    </div>
    <p class="mt-2 text-sm text-zinc-400" id="launch-description">
      Override the launch string. Passed to system's default shell, and replaces
      "{}" with the command to start the game.
      <span class="font-semibold text-zinc-200"
        >Leaving it blank will cause the game not to start.</span
      >
    </p>

    <Listbox
      v-if="props.protonEnabled"
      as="div"
      v-model="model.overrideProtonPath"
      class="mt-6"
    >
      <ListboxLabel class="block text-sm/6 font-medium text-white"
        >Proton override</ListboxLabel
      >
      <div class="relative mt-2">
        <ListboxButton
          class="grid w-full cursor-default grid-cols-1 rounded-md bg-white/5 py-1.5 pr-2 pl-3 text-left text-white outline-1 -outline-offset-1 outline-white/10 focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-blue-500 sm:text-sm/6"
        >
          <span
            v-if="currentProtonPath"
            class="col-start-1 row-start-1 truncate pr-6"
            >{{ currentProtonPath.name }} ({{ currentProtonPath.path }})</span
          >
          <span
            v-else
            class="col-start-1 row-start-1 truncate pr-6 italic text-zinc-400"
            >No override configured</span
          >
          <ChevronUpDownIcon
            class="col-start-1 row-start-1 size-5 self-center justify-self-end text-zinc-400 sm:size-4"
            aria-hidden="true"
          />
        </ListboxButton>

        <transition
          leave-active-class="transition ease-in duration-100"
          leave-from-class=""
          leave-to-class="opacity-0"
        >
          <ListboxOptions
            class="absolute z-10 mt-1 max-h-60 w-full overflow-auto rounded-md bg-zinc-800 py-1 text-base outline-1 -outline-offset-1 outline-white/10 sm:text-sm"
          >
            <ListboxOption
              as="template"
              :value="undefined"
              v-slot="{ active, selected }"
            >
              <li
                :class="[
                  active
                    ? 'bg-blue-500 text-white outline-hidden'
                    : 'text-white',
                  'relative cursor-default py-2 pr-9 pl-3 select-none',
                ]"
              >
                <span
                  :class="[
                    selected ? 'font-semibold' : 'font-normal',
                    'block truncate italic',
                  ]"
                  >Use global default</span
                >

                <span
                  v-if="selected"
                  :class="[
                    active ? 'text-white' : 'text-blue-400',
                    'absolute inset-y-0 right-0 flex items-center pr-4',
                  ]"
                >
                  <CheckIcon class="size-5" aria-hidden="true" />
                </span>
              </li>
            </ListboxOption>
            <h1 class="text-white text-sm font-semibold bg-zinc-900 py-2 px-2">
              Auto-discovered
            </h1>
            <ListboxOption
              as="template"
              v-if="protonPaths.autodiscovered.length > 0"
              v-for="proton in protonPaths.autodiscovered"
              :key="proton.path"
              :value="proton.path"
              v-slot="{ active, selected }"
            >
              <li
                :class="[
                  active
                    ? 'bg-blue-500 text-white outline-hidden'
                    : 'text-white',
                  'relative cursor-default py-2 pr-9 pl-3 select-none',
                ]"
              >
                <span
                  :class="[
                    selected ? 'font-semibold' : 'font-normal',
                    'block truncate',
                  ]"
                  >{{ proton.name }} ({{ proton.path }})</span
                >

                <span
                  v-if="selected"
                  :class="[
                    active ? 'text-white' : 'text-blue-400',
                    'absolute inset-y-0 right-0 flex items-center pr-4',
                  ]"
                >
                  <CheckIcon class="size-5" aria-hidden="true" />
                </span>
              </li>
            </ListboxOption>
            <li v-else class="italic text-zinc-400 py-2 pr-9 pl-3">
              No auto-discovered layers.
            </li>
            <h1 class="text-white text-sm font-semibold bg-zinc-900 py-2 px-2">
              Manually added
            </h1>
            <ListboxOption
              as="template"
              v-if="protonPaths.custom.length > 0"
              v-for="proton in protonPaths.custom"
              :key="proton.path"
              :value="proton.path"
              v-slot="{ active, selected }"
            >
              <li
                :class="[
                  active
                    ? 'bg-blue-500 text-white outline-hidden'
                    : 'text-white',
                  'relative cursor-default py-2 pr-9 pl-3 select-none',
                ]"
              >
                <span
                  :class="[
                    selected ? 'font-semibold' : 'font-normal',
                    'block truncate',
                  ]"
                  >{{ proton.name }} ({{ proton.path }})</span
                >

                <span
                  v-if="selected"
                  :class="[
                    active ? 'text-white' : 'text-blue-400',
                    'absolute inset-y-0 right-0 flex items-center pr-4',
                  ]"
                >
                  <CheckIcon class="size-5" aria-hidden="true" />
                </span>
              </li>
            </ListboxOption>
            <li v-else class="italic text-zinc-400 py-2 pr-9 pl-3">
              No manually added layers.
            </li>
          </ListboxOptions>
        </transition>
      </div>
      <p class="mt-2 text-sm text-zinc-400" id="launch-description">
        Override the Proton layer used to launch this game. You can add or
        remove your custom Proton layer paths in
        <PageWidget to="/settings/compat">
          <WrenchIcon class="size-3" />
          Settings </PageWidget
        >.
      </p>
    </Listbox>
  </div>
</template>

<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import type { ProtonPath } from "~/composables/game";
import {
  Listbox,
  ListboxButton,
  ListboxLabel,
  ListboxOption,
  ListboxOptions,
} from "@headlessui/vue";
import { ChevronUpDownIcon } from "@heroicons/vue/16/solid";
import { CheckIcon } from "@heroicons/vue/20/solid";
import { WrenchIcon } from "@heroicons/vue/24/solid";
import type { GameVersion } from "~/types";

const model = defineModel<GameVersion["userConfiguration"]>({ required: true });

const props = defineProps<{
  protonEnabled: boolean;
}>();

const protonPaths = await invoke<{
  autodiscovered: ProtonPath[];
  custom: ProtonPath[];
  default?: string;
}>("fetch_proton_paths");
const currentProtonPath = computed(
  () =>
    protonPaths.autodiscovered.find(
      (v) => v.path == model.value.overrideProtonPath,
    ) ??
    protonPaths.custom.find((v) => v.path == model.value.overrideProtonPath),
);
</script>
