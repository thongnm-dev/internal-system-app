<script setup lang="ts">
import welcomeBg from "@/assets/welcome.png";
</script>

<template>
  <section class="relative min-h-0 flex-1 overflow-hidden rounded-lg border border-divider shadow-sm">
    <img :src="welcomeBg" alt="" class="absolute inset-0 h-full w-full object-cover" aria-hidden="true" />
  </section>
</template>
