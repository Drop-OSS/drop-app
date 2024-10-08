<template>
  <div
    class="grid min-h-full grid-cols-1 grid-rows-[1fr,auto,1fr] lg:grid-cols-[max(50%,36rem),1fr]"
  >
    <header
      class="mx-auto w-full max-w-7xl px-6 pt-6 sm:pt-10 lg:col-span-2 lg:col-start-1 lg:row-start-1 lg:px-8"
    >
      <Logo class="h-10 w-auto sm:h-12" />
    </header>
    <main
      class="mx-auto w-full max-w-7xl px-6 py-24 sm:py-32 lg:col-span-2 lg:col-start-1 lg:row-start-2 lg:px-8"
    >
      <div class="max-w-lg">
        <h1
          class="mt-4 text-3xl font-bold font-display tracking-tight text-zinc-100 sm:text-5xl"
        >
          Sign in to Drop
        </h1>
        <p class="mt-6 text-base leading-7 text-zinc-400">
          To get started, sign in to your Drop instance by clicking below.
        </p>
        <div class="mt-10">
          <button
            @click="() => auth()"
            :disabled="loading"
            class="text-sm font-semibold leading-7 text-blue-600"
          >
            <div v-if="loading" role="status">
              <svg
                aria-hidden="true"
                class="w-5 h-5 text-transparent animate-spin fill-white"
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
            <span v-else>
              Sign in with your browser <span aria-hidden="true">&rarr;</span>
            </span>
          </button>
        </div>
      </div>
    </main>
    <footer class="self-end lg:col-span-2 lg:col-start-1 lg:row-start-3">
      <div class="border-t border-blue-600 bg-zinc-900 py-10">
        <nav
          class="mx-auto flex w-full max-w-7xl items-center gap-x-4 px-6 text-sm leading-7 text-zinc-400 lg:px-8"
        >
          <a href="#">Documentation</a>
          <svg
            viewBox="0 0 2 2"
            aria-hidden="true"
            class="h-0.5 w-0.5 fill-zinc-700"
          >
            <circle cx="1" cy="1" r="1" />
          </svg>
          <a href="#">Troubleshooting</a>
          <svg
            viewBox="0 0 2 2"
            aria-hidden="true"
            class="h-0.5 w-0.5 fill-zinc-700"
          >
            <circle cx="1" cy="1" r="1" />
          </svg>
          <NuxtLink to="/setup/server">Switch instance</NuxtLink>
        </nav>
      </div>
    </footer>
    <div
      class="hidden lg:relative lg:col-start-2 lg:row-start-1 lg:row-end-4 lg:block"
    >
      <img
        src="@/assets/wallpaper.jpg"
        alt=""
        class="absolute inset-0 h-full w-full object-cover"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";

const loading = ref(false);

async function auth() {
  loading.value = true;
  await invoke("auth_initiate");
}

definePageMeta({
  layout: "mini",
});
</script>
