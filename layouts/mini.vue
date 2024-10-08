<template>
  <div class="flex flex-col bg-zinc-950 h-screen w-screen overflow-hidden">
    <MiniHeader />
    <div class="grow overflow-y-scroll">
      <slot />
    </div>
  </div>
</template>
