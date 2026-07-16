<script setup lang="ts">
import { useExcel2md } from "../composables/useExcel2md";
import MessageBanner from "@/shared/components/MessageBanner.vue";

const ctrl = useExcel2md();
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
    <div class="grid grid-cols-[minmax(0,1fr)_320px] gap-4">
      <section class="rounded-lg border border-divider bg-panel p-4 shadow-sm">
        <div class="flex items-center gap-2">
          <i class="pi pi-file-excel text-xl text-brand" />
          <h3 class="font-bold">Excel workbook</h3>
        </div>

        <div class="mt-4 grid gap-3">
          <label class="grid gap-1.5">
            <span class="text-xs font-bold uppercase tracking-wide text-muted">Input .xlsx</span>
            <div class="grid grid-cols-[minmax(0,1fr)_auto] gap-2">
              <input
                class="h-10 min-w-0 rounded-md border border-divider bg-panel px-3 text-sm outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                placeholder="Select Excel workbook..."
                type="text"
                :value="ctrl.inputPath.value"
                @input="ctrl.updateInputPath(($event.target as HTMLInputElement).value)"
              />
              <button
                class="flex h-10 w-10 items-center justify-center rounded-md border border-divider bg-panel text-secondary hover:bg-canvas"
                type="button"
                title="Browse Excel workbook"
                @click="ctrl.pickInputFile()"
              >
                <i class="pi pi-folder-open" />
              </button>
            </div>
          </label>

          <label class="grid gap-1.5">
            <span class="text-xs font-bold uppercase tracking-wide text-muted">Output .md</span>
            <div class="grid grid-cols-[minmax(0,1fr)_auto] gap-2">
              <input
                class="h-10 min-w-0 rounded-md border border-divider bg-panel px-3 text-sm outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                placeholder="Markdown output path..."
                type="text"
                :value="ctrl.outputPath.value"
                @input="ctrl.setOutputPath(($event.target as HTMLInputElement).value)"
              />
              <button
                class="flex h-10 w-10 items-center justify-center rounded-md border border-divider bg-panel text-secondary hover:bg-canvas"
                type="button"
                title="Choose Markdown output"
                @click="ctrl.pickOutputFile()"
              >
                <i class="pi pi-save" />
              </button>
            </div>
          </label>

          <button
            class="inline-flex h-10 w-fit items-center justify-center gap-2 rounded-md bg-brand px-4 text-sm font-bold text-white hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-60"
            type="button"
            :disabled="ctrl.isConverting.value"
            @click="ctrl.convert()"
          >
            <i class="pi pi-bolt" />
            {{ ctrl.isConverting.value ? "Converting..." : "Convert" }}
          </button>
        </div>
      </section>

      <aside class="rounded-lg border border-divider bg-panel p-4 shadow-sm">
        <span class="text-sm font-bold text-muted">Last conversion</span>
        <strong class="mt-2 block truncate text-lg leading-tight text-ink">
          {{ ctrl.result.value?.output_file_name ?? "-" }}
        </strong>
        <p class="mt-2 text-xs text-muted">
          {{ ctrl.result.value ? ctrl.result.value.output_path : "No Markdown file created yet." }}
        </p>
        <div v-if="ctrl.result.value" class="mt-4 flex items-center gap-2 text-sm font-bold text-brand">
          <i class="pi pi-check-circle" />
          Ready
        </div>
      </aside>
    </div>

    <MessageBanner :message="ctrl.message.value" :mode="ctrl.messageMode.value" />

    <section class="flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border border-divider bg-panel shadow-sm">
      <div class="flex items-center justify-between gap-3 border-b border-divider px-4 py-3">
        <div class="flex min-w-0 items-center gap-2">
          <i class="pi pi-code text-brand" />
          <h3 class="font-bold">Markdown preview</h3>
        </div>
        <span class="truncate text-xs font-semibold text-muted">
          {{ ctrl.result.value?.source_file_name ?? "No file selected" }}
        </span>
      </div>
      <pre class="min-h-0 flex-1 overflow-auto whitespace-pre-wrap break-words bg-slate-950 p-4 text-sm leading-6 text-slate-100">{{ ctrl.result.value?.markdown || "Converted Markdown will appear here." }}</pre>
    </section>
  </section>
</template>
