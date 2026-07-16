<script setup lang="ts">
import { ref } from "vue";

const search = ref("");
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
    <section class="rounded-lg border border-divider bg-panel p-4 shadow-sm">
      <div class="flex items-center gap-2">
        <i class="pi pi-compass text-xl text-brand" />
        <h3 class="font-bold">Explore Faster</h3>
      </div>

      <div class="mt-4 grid gap-3">
        <label class="grid gap-1.5">
          <span class="text-xs font-bold uppercase tracking-wide text-muted">Search</span>
          <div class="relative">
            <i class="pi pi-search pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-muted" />
            <input
              v-model="search"
              class="h-10 w-full rounded-md border border-divider bg-panel pl-9 pr-3 text-sm outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
              placeholder="Search projects, tasks, tables..."
              type="text"
            />
          </div>
        </label>
      </div>
    </section>

    <section class="flex min-h-0 flex-1 flex-col rounded-lg border border-divider bg-panel p-4 shadow-sm">
      <div class="flex flex-1 items-center justify-center text-sm text-muted">
        Start typing to explore your data.
      </div>
    </section>
  </section>
</template>
