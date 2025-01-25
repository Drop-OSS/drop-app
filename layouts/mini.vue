<template>
  <div class="flex flex-col bg-zinc-950 overflow-hidden">
    <MiniHeader />
    <div class="relative grow overflow-y-auto">
      <slot />
    </div>
  </div>
</template>
