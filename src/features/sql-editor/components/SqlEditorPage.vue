<script setup lang="ts">
import { ref } from "vue";

const query = ref("SELECT * FROM projects LIMIT 100;");
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
    <section class="rounded-lg border border-divider bg-panel p-4 shadow-sm">
      <div class="flex items-center gap-2">
        <i class="pi pi-server text-xl text-brand" />
        <h3 class="font-bold">SQL Editor</h3>
      </div>

      <div class="mt-4 grid gap-3">
        <label class="grid gap-1.5">
          <span class="text-xs font-bold uppercase tracking-wide text-muted">Query</span>
          <textarea
            v-model="query"
            class="min-h-[180px] rounded-md border border-divider bg-panel p-3 font-mono text-sm outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
            spellcheck="false"
          />
        </label>

        <div class="flex justify-end">
          <button
            class="flex h-10 items-center gap-2 rounded-md bg-brand px-4 text-sm font-semibold text-white hover:opacity-90"
            type="button"
          >
            <i class="pi pi-play" />
            Run query
          </button>
        </div>
      </div>
    </section>

    <section class="flex min-h-0 flex-1 flex-col rounded-lg border border-divider bg-panel p-4 shadow-sm">
      <h3 class="font-bold">Results</h3>
      <div class="mt-3 flex flex-1 items-center justify-center text-sm text-muted">
        Run a query to see results.
      </div>
    </section>
  </section>
</template>
