<template>
  <div class="flex flex-col bg-zinc-950 overflow-hidden">
    <MiniHeader />
    <div class="grow overflow-y-scroll">
      <slot />
    </div>
  </div>
</template>
