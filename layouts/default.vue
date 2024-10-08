<template>
  <div class="flex flex-col bg-zinc-900 h-screen w-screen overflow-hidden">
    <Header class="select-none" />
    <div class="grow overflow-y-scroll">
      <slot />
    </div>
  </div>
</template>
