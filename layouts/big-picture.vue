<template>
  <div class="flex h-screen w-screen bg-zinc-950 overflow-hidden">
    <!-- Sidebar -->
    <BigPictureSidebar class="w-80 flex-shrink-0" />
    
    <!-- Main Content Area -->
    <div class="flex flex-col flex-1 min-w-0">
      <!-- Header -->
      <BigPictureHeader class="h-20 flex-shrink-0" />
      
      <!-- Content -->
      <div class="flex-1 overflow-hidden">
        <div class="h-full overflow-y-auto">
          <slot />
        </div>
      </div>
    </div>
  </div>
</template>
