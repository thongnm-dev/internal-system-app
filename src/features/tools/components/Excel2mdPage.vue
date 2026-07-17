<script setup lang="ts">
import Button from "primevue/button";
import InputText from "primevue/inputtext";
import { useExcel2md } from "../composables/useExcel2md";

const ctrl = useExcel2md();
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
    <div class="grid grid-cols-1 gap-4">
      <section class="rounded-lg border border-divider bg-panel p-4 shadow-sm">
        <div class="flex items-center gap-2">
          <i class="pi pi-file-excel text-xl text-brand" />
          <h3 class="font-bold">Excel workbook</h3>
        </div>

        <div class="mt-4 grid gap-3">
          <label class="grid gap-1.5">
            <span class="text-xs font-bold uppercase tracking-wide text-muted">Input .xlsx</span>
            <div class="grid grid-cols-[minmax(0,1fr)_auto] gap-2">
              <InputText
                class="h-10 min-w-0"
                placeholder="Select Excel workbook..."
                :model-value="ctrl.inputPath.value"
                @update:model-value="ctrl.updateInputPath($event as string)"
              />
              <Button icon="pi pi-folder-open" severity="secondary" outlined title="Browse Excel workbook" @click="ctrl.pickInputFile()" />
            </div>
          </label>

          <label class="grid gap-1.5">
            <span class="text-xs font-bold uppercase tracking-wide text-muted">Output .md</span>
            <div class="grid grid-cols-[minmax(0,1fr)_auto] gap-2">
              <InputText
                class="h-10 min-w-0"
                placeholder="Markdown output path..."
                :model-value="ctrl.outputPath.value"
                @update:model-value="ctrl.setOutputPath($event as string)"
              />
              <Button icon="pi pi-save" severity="secondary" outlined title="Choose Markdown output" @click="ctrl.pickOutputFile()" />
            </div>
          </label>

          <Button
            icon="pi pi-bolt"
            :label="ctrl.isConverting.value ? 'Converting...' : 'Convert'"
            :disabled="ctrl.isConverting.value"
            @click="ctrl.convert()"
          />
        </div>
      </section>
    </div>

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
