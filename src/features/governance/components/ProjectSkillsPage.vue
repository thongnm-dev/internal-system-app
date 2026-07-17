<script setup lang="ts">
import { ref } from "vue";
import Button from "primevue/button";
import Fieldset from "primevue/fieldset";
import InputNumber from "primevue/inputnumber";
import InputText from "primevue/inputtext";
import MessageBanner from "@/shared/components/MessageBanner.vue";
import {
  skillCategories,
  useProjectSkills,
  type ManagedSkill,
  type SkillCategory,
  type SkillSortKey,
  type SkillViewMode,
} from "../composables/useProjectSkills";

const ctrl = useProjectSkills();

const sortOptions: { label: string; value: SkillSortKey }[] = [
  { label: "Featured", value: "featured" },
  { label: "Most downloaded", value: "downloads" },
  { label: "Most starred", value: "stars" },
  { label: "Most installed", value: "installs" },
  { label: "Recently updated", value: "updated" },
  { label: "Newest", value: "newest" },
  { label: "Name", value: "name" },
];

const statusOptions = ["Active", "Draft", "Deprecated"] as const;
const categoryOptions = ["All", ...skillCategories] as const;

const copyMessage = ref("");

async function copyMarkdown() {
  try {
    await navigator.clipboard.writeText(ctrl.generatedMarkdown.value);
    copyMessage.value = "Generated skill markdown copied to clipboard.";
  } catch {
    copyMessage.value = "Clipboard is unavailable in this runtime.";
  }
}

function confirmDelete() {
  const skillLabel = ctrl.draft.value.name || "this unsaved skill";
  if (window.confirm(`Delete ${skillLabel}?`)) {
    ctrl.deleteDraft();
  }
}

function updateTags(value: string) {
  ctrl.updateDraft(
    "tags",
    value.split(",").map((t) => t.trim()).filter(Boolean),
  );
}

function statusBadgeClass(status: string): string {
  if (status === "Active") return "bg-emerald-50 text-emerald-700";
  if (status === "Deprecated") return "bg-red-50 text-red-700";
  return "bg-amber-50 text-amber-700";
}

function skillCardClass(skill: ManagedSkill, isActive: boolean, viewMode: SkillViewMode): string {
  return [
    "w-full rounded-md border bg-panel p-4 text-left transition cursor-pointer",
    isActive ? "border-brand ring-2 ring-emerald-100" : "border-divider hover:bg-canvas",
    viewMode === "list" ? "flex items-start justify-between gap-4" : "block min-h-52",
  ].join(" ");
}
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
    <!-- Top bar: search, sort, stats, new -->
    <section class="grid gap-3 rounded-lg border border-divider bg-panel p-4 shadow-sm xl:grid-cols-[minmax(260px,1fr)_auto]">
      <div class="grid gap-3 lg:grid-cols-[minmax(220px,1fr)_220px]">
        <label class="block min-w-0">
          <span class="text-xs font-bold text-muted">Search Skills</span>
          <span class="mt-1 flex h-10 items-center gap-2 rounded-md border border-divider bg-panel px-3 focus-within:border-brand focus-within:ring-2 focus-within:ring-emerald-100">
            <i class="pi pi-search shrink-0 text-muted" />
            <InputText
              class="min-w-0 flex-1 border-0 bg-transparent p-0 text-sm text-ink outline-none shadow-none"
              placeholder="Name, tag, category, guidance"
              :model-value="ctrl.query.value"
              @update:model-value="ctrl.query.value = $event as string"
            />
          </span>
        </label>
        <label class="block min-w-0">
          <span class="text-xs font-bold text-muted">Sort by</span>
          <select
            class="mt-1 flex h-10 w-full items-center rounded-md border border-divider bg-panel px-3 text-sm"
            :value="ctrl.sortKey.value"
            @change="ctrl.sortKey.value = ($event.target as HTMLSelectElement).value as SkillSortKey"
          >
            <option v-for="opt in sortOptions" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
          </select>
        </label>
      </div>

      <div class="flex flex-wrap items-end justify-between gap-2 xl:justify-end">
        <Button icon="pi pi-plus" label="New" title="Create skill" @click="ctrl.createSkill()" />
      </div>
    </section>

    <!-- Category tabs + view mode -->
    <section class="flex flex-wrap items-center justify-between gap-3">
      <div class="flex flex-wrap gap-2">
        <Button
          v-for="item in categoryOptions"
          :key="item"
          :label="item"
          :class="[
            'h-9 text-xs',
            ctrl.category.value === item
              ? 'border-brand bg-emerald-50 text-brand'
              : '',
          ]"
          :severity="ctrl.category.value === item ? undefined : 'secondary'"
          :outlined="ctrl.category.value !== item"
          size="small"
          @click="ctrl.category.value = item as SkillCategory | 'All'"
        />
      </div>
      <div class="grid h-9 grid-cols-2 gap-1 rounded-md border border-divider bg-panel p-1">
        <Button
          icon="pi pi-list"
          :class="['flex h-7 items-center justify-center rounded px-2', ctrl.viewMode.value === 'list' ? 'bg-canvas' : '']"
          unstyled
          @click="ctrl.viewMode.value = 'list'"
        />
        <Button
          icon="pi pi-th-large"
          :class="['flex h-7 items-center justify-center rounded px-2', ctrl.viewMode.value === 'grid' ? 'bg-canvas' : '']"
          unstyled
          @click="ctrl.viewMode.value = 'grid'"
        />
      </div>
    </section>

    <MessageBanner :message="ctrl.message.value" :mode="ctrl.messageMode.value" />

    <!-- Main: skill list + detail editor -->
    <section class="grid min-h-0 flex-1 gap-4 overflow-hidden xl:grid-cols-[minmax(360px,1fr)_minmax(420px,520px)]">
      <!-- Skill list -->
      <section class="min-h-0 overflow-auto rounded-lg border border-divider bg-panel p-3 shadow-sm">
        <p v-if="ctrl.filteredSkills.value.length === 0" class="p-3 text-sm text-muted">No skills match the current filters.</p>
        <div v-else :class="ctrl.viewMode.value === 'grid' ? 'grid gap-3 2xl:grid-cols-2' : 'grid gap-2'">
          <Button
            v-for="skill in ctrl.filteredSkills.value"
            :key="skill.id"
            :class="skillCardClass(skill, ctrl.selectedSkillId.value === skill.id, ctrl.viewMode.value)"
            unstyled
            @click="ctrl.selectSkill(skill.id)"
          >
            <span class="min-w-0 flex-1">
              <span class="flex min-w-0 flex-wrap items-center gap-2">
                <span class="min-w-0 truncate text-base font-extrabold text-ink">{{ skill.name || "Untitled Skill" }}</span>
                <span class="shrink-0 rounded-md border border-divider px-2 py-0.5 text-[11px] font-bold text-muted">{{ skill.category }}</span>
                <span :class="['shrink-0 rounded-md px-2 py-0.5 text-[11px] font-bold', statusBadgeClass(skill.status)]">{{ skill.status }}</span>
              </span>
              <span class="mt-2 block text-sm font-normal leading-6 text-secondary">{{ skill.description || "No description" }}</span>
              <span class="mt-3 flex flex-wrap gap-1">
                <span v-for="tag in skill.tags.slice(0, 4)" :key="tag" class="rounded-md bg-canvas px-2 py-1 text-[11px] font-bold text-muted">{{ tag }}</span>
              </span>
            </span>
            <span :class="ctrl.viewMode.value === 'list' ? 'grid shrink-0 gap-2 text-right' : 'mt-4 flex gap-4'">
              <span class="flex items-center justify-end gap-1 text-xs font-bold text-muted">
                <i class="pi pi-download text-sm" />
                {{ skill.downloads.toLocaleString("en-US") }}
              </span>
              <span class="flex items-center justify-end gap-1 text-xs font-bold text-muted">
                <i class="pi pi-star text-sm" />
                {{ skill.stars.toLocaleString("en-US") }}
              </span>
            </span>
          </Button>
        </div>
      </section>

      <!-- Detail editor -->
      <section class="flex min-h-0 flex-col overflow-hidden rounded-lg border border-divider bg-panel shadow-sm">
        <div class="flex items-center justify-between gap-3 border-b border-divider px-4 py-3">
          <h3 class="font-bold">Skill Details</h3>
          <div class="flex items-center gap-2">
            <Button icon="pi pi-refresh" label="Reset" severity="secondary" outlined size="small" title="Reset draft" @click="ctrl.resetDraft()" />
            <Button icon="pi pi-save" label="Save" size="small" title="Save skill" @click="ctrl.saveDraft()" />
          </div>
        </div>

        <div class="min-h-0 flex-1 overflow-auto p-4">
          <Fieldset class="rounded-lg border border-divider bg-panel p-4 shadow-md fieldset-nested" legend="Metadata" toggleable>
            <div class="grid gap-3 md:grid-cols-2">
              <label class="block min-w-0 md:col-span-2">
                <span class="text-xs font-bold text-muted">Skill Name</span>
                <InputText
                  class="mt-1 w-full"
                  placeholder="Skill name"
                  :model-value="ctrl.draft.value.name"
                  @update:model-value="ctrl.updateDraft('name', $event as string)"
                />
              </label>
              <label class="block min-w-0">
                <span class="text-xs font-bold text-muted">Category</span>
                <select
                  class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm"
                  :value="ctrl.draft.value.category"
                  @change="ctrl.updateDraft('category', ($event.target as HTMLSelectElement).value as any)"
                >
                  <option v-for="cat in skillCategories" :key="cat" :value="cat">{{ cat }}</option>
                </select>
              </label>
              <label class="block min-w-0">
                <span class="text-xs font-bold text-muted">Status</span>
                <select
                  class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm"
                  :value="ctrl.draft.value.status"
                  @change="ctrl.updateDraft('status', ($event.target as HTMLSelectElement).value as any)"
                >
                  <option v-for="s in statusOptions" :key="s" :value="s">{{ s }}</option>
                </select>
              </label>
              <label class="block min-w-0">
                <span class="text-xs font-bold text-muted">Publisher</span>
                <InputText
                  class="mt-1 w-full"
                  placeholder="Publisher"
                  :model-value="ctrl.draft.value.publisher"
                  @update:model-value="ctrl.updateDraft('publisher', $event as string)"
                />
              </label>
              <label class="block min-w-0">
                <span class="text-xs font-bold text-muted">Version</span>
                <InputText
                  class="mt-1 w-full"
                  placeholder="0.1.0"
                  :model-value="ctrl.draft.value.version"
                  @update:model-value="ctrl.updateDraft('version', $event as string)"
                />
              </label>
              <label class="block min-w-0 md:col-span-2">
                <span class="text-xs font-bold text-muted">Description</span>
                <textarea
                  class="mt-1 min-h-20 w-full resize-y rounded-md border border-divider bg-panel px-3 py-2 text-sm leading-6 text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                  placeholder="What this skill does"
                  :value="ctrl.draft.value.description"
                  @input="ctrl.updateDraft('description', ($event.target as HTMLTextAreaElement).value)"
                />
              </label>
              <label class="block min-w-0 md:col-span-2">
                <span class="text-xs font-bold text-muted">Tags</span>
                <InputText
                  class="mt-1 w-full"
                  placeholder="browser, qa, frontend"
                  :model-value="ctrl.draft.value.tags.join(', ')"
                  @update:model-value="updateTags($event as string)"
                />
              </label>
            </div>
          </Fieldset>

          <Fieldset class="mt-4 rounded-lg border border-divider bg-panel p-4 shadow-md fieldset-nested" legend="Guidance" toggleable>
            <label class="block min-w-0">
              <span class="text-xs font-bold text-muted">When To Use</span>
              <textarea
                class="mt-1 min-h-24 w-full resize-y rounded-md border border-divider bg-panel px-3 py-2 text-sm leading-6 text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                placeholder="Use when..."
                :value="ctrl.draft.value.usage"
                @input="ctrl.updateDraft('usage', ($event.target as HTMLTextAreaElement).value)"
              />
            </label>
            <label class="mt-3 block min-w-0">
              <span class="text-xs font-bold text-muted">Operational Guidance</span>
              <textarea
                class="mt-1 min-h-32 w-full resize-y rounded-md border border-divider bg-panel px-3 py-2 text-sm leading-6 text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                placeholder="Step-by-step behavior, constraints, and verification"
                :value="ctrl.draft.value.guidance"
                @input="ctrl.updateDraft('guidance', ($event.target as HTMLTextAreaElement).value)"
              />
            </label>
          </Fieldset>

          <Fieldset class="mt-4 rounded-lg border border-divider bg-panel p-4 shadow-md fieldset-nested" legend="Catalog Metrics" toggleable :collapsed="true">
            <div class="grid gap-3 md:grid-cols-3">
              <label class="block min-w-0">
                <span class="text-xs font-bold text-muted">Downloads</span>
                <InputNumber
                  class="mt-1 w-full"
                  :model-value="ctrl.draft.value.downloads"
                  :min="0"
                  :useGrouping="false"
                  @update:model-value="ctrl.updateDraft('downloads', $event ?? 0)"
                />
              </label>
              <label class="block min-w-0">
                <span class="text-xs font-bold text-muted">Stars</span>
                <InputNumber
                  class="mt-1 w-full"
                  :model-value="ctrl.draft.value.stars"
                  :min="0"
                  :useGrouping="false"
                  @update:model-value="ctrl.updateDraft('stars', $event ?? 0)"
                />
              </label>
              <label class="block min-w-0">
                <span class="text-xs font-bold text-muted">Installs</span>
                <InputNumber
                  class="mt-1 w-full"
                  :model-value="ctrl.draft.value.installs"
                  :min="0"
                  :useGrouping="false"
                  @update:model-value="ctrl.updateDraft('installs', $event ?? 0)"
                />
              </label>
            </div>
          </Fieldset>

          <!-- Generated Markdown -->
          <section class="mt-4 flex min-h-0 flex-col overflow-hidden rounded-lg border border-divider bg-panel">
            <div class="flex items-center justify-between gap-3 border-b border-divider px-4 py-3">
              <h3 class="font-bold">Generated Markdown</h3>
              <Button icon="pi pi-copy" label="Copy" severity="secondary" outlined size="small" title="Copy generated markdown" @click="copyMarkdown()" />
            </div>
            <p v-if="copyMessage" class="border-b border-divider px-4 py-2 text-xs text-muted">{{ copyMessage }}</p>
            <pre class="max-h-72 overflow-auto whitespace-pre-wrap p-4 font-mono text-xs leading-5 text-ink">{{ ctrl.generatedMarkdown.value }}</pre>
          </section>

          <div class="mt-4 flex justify-end">
            <Button icon="pi pi-trash" label="Delete" severity="danger" outlined size="small" title="Delete skill" @click="confirmDelete" />
          </div>
        </div>
      </section>
    </section>
  </section>
</template>
