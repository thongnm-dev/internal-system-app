<script setup lang="ts">
import { ref } from "vue";
import Fieldset from "primevue/fieldset";
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
    "w-full rounded-md border bg-white p-4 text-left transition cursor-pointer",
    isActive ? "border-brand ring-2 ring-emerald-100" : "border-slate-200 hover:bg-slate-50",
    viewMode === "list" ? "flex items-start justify-between gap-4" : "block min-h-52",
  ].join(" ");
}
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
    <!-- Top bar: search, sort, stats, new -->
    <section class="grid gap-3 rounded-lg border border-stone-200 bg-panel p-4 shadow-sm xl:grid-cols-[minmax(260px,1fr)_auto]">
      <div class="grid gap-3 lg:grid-cols-[minmax(220px,1fr)_220px]">
        <label class="block min-w-0">
          <span class="text-xs font-bold text-slate-500">Search Skills</span>
          <span class="mt-1 flex h-10 items-center gap-2 rounded-md border border-slate-300 bg-white px-3 focus-within:border-brand focus-within:ring-2 focus-within:ring-emerald-100">
            <i class="pi pi-search shrink-0 text-slate-400" />
            <input
              class="min-w-0 flex-1 border-0 bg-transparent p-0 text-sm text-slate-900 outline-none shadow-none"
              placeholder="Name, tag, category, guidance"
              type="search"
              :value="ctrl.query.value"
              @input="ctrl.query.value = ($event.target as HTMLInputElement).value"
            />
          </span>
        </label>
        <label class="block min-w-0">
          <span class="text-xs font-bold text-slate-500">Sort by</span>
          <select
            class="mt-1 flex h-10 w-full items-center rounded-md border border-slate-300 bg-white px-3 text-sm"
            :value="ctrl.sortKey.value"
            @change="ctrl.sortKey.value = ($event.target as HTMLSelectElement).value as SkillSortKey"
          >
            <option v-for="opt in sortOptions" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
          </select>
        </label>
      </div>

      <div class="flex flex-wrap items-end justify-between gap-2 xl:justify-end">
        <span class="min-w-20 rounded-md border border-slate-200 bg-white px-3 py-2 text-right">
          <span class="block text-[11px] font-bold uppercase text-slate-500">Skills</span>
          <span class="block text-sm font-extrabold text-slate-900">{{ ctrl.stats.value.total.toLocaleString("en-US") }}</span>
        </span>
        <span class="min-w-20 rounded-md border border-slate-200 bg-white px-3 py-2 text-right">
          <span class="block text-[11px] font-bold uppercase text-slate-500">Active</span>
          <span class="block text-sm font-extrabold text-slate-900">{{ ctrl.stats.value.active.toLocaleString("en-US") }}</span>
        </span>
        <span class="min-w-20 rounded-md border border-slate-200 bg-white px-3 py-2 text-right">
          <span class="block text-[11px] font-bold uppercase text-slate-500">Drafts</span>
          <span class="block text-sm font-extrabold text-slate-900">{{ ctrl.stats.value.draft.toLocaleString("en-US") }}</span>
        </span>
        <button
          class="flex h-10 items-center gap-2 rounded-md bg-brand px-4 text-sm font-bold text-white hover:opacity-90"
          type="button"
          title="Create skill"
          @click="ctrl.createSkill()"
        >
          <i class="pi pi-plus" />
          New
        </button>
      </div>
    </section>

    <!-- Category tabs + view mode -->
    <section class="flex flex-wrap items-center justify-between gap-3">
      <div class="flex flex-wrap gap-2">
        <button
          v-for="item in categoryOptions"
          :key="item"
          :class="[
            'h-9 rounded-md border px-3 text-xs font-bold',
            ctrl.category.value === item
              ? 'border-brand bg-emerald-50 text-brand'
              : 'border-slate-300 bg-white text-slate-600 hover:bg-slate-50',
          ]"
          type="button"
          @click="ctrl.category.value = item as SkillCategory | 'All'"
        >
          {{ item }}
        </button>
      </div>
      <div class="grid h-9 grid-cols-2 gap-1 rounded-md border border-slate-300 bg-white p-1">
        <button
          :class="['flex h-7 items-center justify-center rounded px-2', ctrl.viewMode.value === 'list' ? 'bg-slate-100' : '']"
          type="button"
          @click="ctrl.viewMode.value = 'list'"
        >
          <i class="pi pi-list" />
        </button>
        <button
          :class="['flex h-7 items-center justify-center rounded px-2', ctrl.viewMode.value === 'grid' ? 'bg-slate-100' : '']"
          type="button"
          @click="ctrl.viewMode.value = 'grid'"
        >
          <i class="pi pi-th-large" />
        </button>
      </div>
    </section>

    <MessageBanner :message="ctrl.message.value" :mode="ctrl.messageMode.value" />

    <!-- Main: skill list + detail editor -->
    <section class="grid min-h-0 flex-1 gap-4 overflow-hidden xl:grid-cols-[minmax(360px,1fr)_minmax(420px,520px)]">
      <!-- Skill list -->
      <section class="min-h-0 overflow-auto rounded-lg border border-stone-200 bg-panel p-3 shadow-sm">
        <p v-if="ctrl.filteredSkills.value.length === 0" class="p-3 text-sm text-slate-500">No skills match the current filters.</p>
        <div v-else :class="ctrl.viewMode.value === 'grid' ? 'grid gap-3 2xl:grid-cols-2' : 'grid gap-2'">
          <button
            v-for="skill in ctrl.filteredSkills.value"
            :key="skill.id"
            :class="skillCardClass(skill, ctrl.selectedSkillId.value === skill.id, ctrl.viewMode.value)"
            type="button"
            @click="ctrl.selectSkill(skill.id)"
          >
            <span class="min-w-0 flex-1">
              <span class="flex min-w-0 flex-wrap items-center gap-2">
                <span class="min-w-0 truncate text-base font-extrabold text-slate-900">{{ skill.name || "Untitled Skill" }}</span>
                <span class="shrink-0 rounded-md border border-slate-200 px-2 py-0.5 text-[11px] font-bold text-slate-500">{{ skill.category }}</span>
                <span :class="['shrink-0 rounded-md px-2 py-0.5 text-[11px] font-bold', statusBadgeClass(skill.status)]">{{ skill.status }}</span>
              </span>
              <span class="mt-2 block text-sm font-normal leading-6 text-slate-600">{{ skill.description || "No description" }}</span>
              <span class="mt-3 flex flex-wrap gap-1">
                <span v-for="tag in skill.tags.slice(0, 4)" :key="tag" class="rounded-md bg-slate-100 px-2 py-1 text-[11px] font-bold text-slate-500">{{ tag }}</span>
              </span>
            </span>
            <span :class="ctrl.viewMode.value === 'list' ? 'grid shrink-0 gap-2 text-right' : 'mt-4 flex gap-4'">
              <span class="flex items-center justify-end gap-1 text-xs font-bold text-slate-500">
                <i class="pi pi-download text-sm" />
                {{ skill.downloads.toLocaleString("en-US") }}
              </span>
              <span class="flex items-center justify-end gap-1 text-xs font-bold text-slate-500">
                <i class="pi pi-star text-sm" />
                {{ skill.stars.toLocaleString("en-US") }}
              </span>
            </span>
          </button>
        </div>
      </section>

      <!-- Detail editor -->
      <section class="flex min-h-0 flex-col overflow-hidden rounded-lg border border-stone-200 bg-panel shadow-sm">
        <div class="flex items-center justify-between gap-3 border-b border-stone-200 px-4 py-3">
          <h3 class="font-bold">Skill Details</h3>
          <div class="flex items-center gap-2">
            <button
              class="flex h-9 items-center gap-2 rounded-md border border-slate-300 bg-white px-3 text-sm font-bold text-slate-600 hover:bg-slate-50"
              type="button"
              title="Reset draft"
              @click="ctrl.resetDraft()"
            >
              <i class="pi pi-refresh" />
              Reset
            </button>
            <button
              class="flex h-9 items-center gap-2 rounded-md bg-brand px-3 text-sm font-bold text-white hover:opacity-90"
              type="button"
              title="Save skill"
              @click="ctrl.saveDraft()"
            >
              <i class="pi pi-save" />
              Save
            </button>
          </div>
        </div>

        <div class="min-h-0 flex-1 overflow-auto p-4">
          <Fieldset class="rounded-lg border border-slate-200 bg-white p-4 shadow-md fieldset-nested" legend="Metadata" toggleable>
            <div class="grid gap-3 md:grid-cols-2">
              <label class="block min-w-0 md:col-span-2">
                <span class="text-xs font-bold text-slate-500">Skill Name</span>
                <input
                  class="mt-1 h-10 w-full rounded-md border border-slate-300 bg-white px-3 text-sm text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                  placeholder="Skill name"
                  :value="ctrl.draft.value.name"
                  @input="ctrl.updateDraft('name', ($event.target as HTMLInputElement).value)"
                />
              </label>
              <label class="block min-w-0">
                <span class="text-xs font-bold text-slate-500">Category</span>
                <select
                  class="mt-1 h-10 w-full rounded-md border border-slate-300 bg-white px-3 text-sm"
                  :value="ctrl.draft.value.category"
                  @change="ctrl.updateDraft('category', ($event.target as HTMLSelectElement).value as any)"
                >
                  <option v-for="cat in skillCategories" :key="cat" :value="cat">{{ cat }}</option>
                </select>
              </label>
              <label class="block min-w-0">
                <span class="text-xs font-bold text-slate-500">Status</span>
                <select
                  class="mt-1 h-10 w-full rounded-md border border-slate-300 bg-white px-3 text-sm"
                  :value="ctrl.draft.value.status"
                  @change="ctrl.updateDraft('status', ($event.target as HTMLSelectElement).value as any)"
                >
                  <option v-for="s in statusOptions" :key="s" :value="s">{{ s }}</option>
                </select>
              </label>
              <label class="block min-w-0">
                <span class="text-xs font-bold text-slate-500">Publisher</span>
                <input
                  class="mt-1 h-10 w-full rounded-md border border-slate-300 bg-white px-3 text-sm text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                  placeholder="Publisher"
                  :value="ctrl.draft.value.publisher"
                  @input="ctrl.updateDraft('publisher', ($event.target as HTMLInputElement).value)"
                />
              </label>
              <label class="block min-w-0">
                <span class="text-xs font-bold text-slate-500">Version</span>
                <input
                  class="mt-1 h-10 w-full rounded-md border border-slate-300 bg-white px-3 text-sm text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                  placeholder="0.1.0"
                  :value="ctrl.draft.value.version"
                  @input="ctrl.updateDraft('version', ($event.target as HTMLInputElement).value)"
                />
              </label>
              <label class="block min-w-0 md:col-span-2">
                <span class="text-xs font-bold text-slate-500">Description</span>
                <textarea
                  class="mt-1 min-h-20 w-full resize-y rounded-md border border-slate-300 bg-white px-3 py-2 text-sm leading-6 text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                  placeholder="What this skill does"
                  :value="ctrl.draft.value.description"
                  @input="ctrl.updateDraft('description', ($event.target as HTMLTextAreaElement).value)"
                />
              </label>
              <label class="block min-w-0 md:col-span-2">
                <span class="text-xs font-bold text-slate-500">Tags</span>
                <input
                  class="mt-1 h-10 w-full rounded-md border border-slate-300 bg-white px-3 text-sm text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                  placeholder="browser, qa, frontend"
                  :value="ctrl.draft.value.tags.join(', ')"
                  @input="updateTags(($event.target as HTMLInputElement).value)"
                />
              </label>
            </div>
          </Fieldset>

          <Fieldset class="mt-4 rounded-lg border border-slate-200 bg-white p-4 shadow-md fieldset-nested" legend="Guidance" toggleable>
            <label class="block min-w-0">
              <span class="text-xs font-bold text-slate-500">When To Use</span>
              <textarea
                class="mt-1 min-h-24 w-full resize-y rounded-md border border-slate-300 bg-white px-3 py-2 text-sm leading-6 text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                placeholder="Use when..."
                :value="ctrl.draft.value.usage"
                @input="ctrl.updateDraft('usage', ($event.target as HTMLTextAreaElement).value)"
              />
            </label>
            <label class="mt-3 block min-w-0">
              <span class="text-xs font-bold text-slate-500">Operational Guidance</span>
              <textarea
                class="mt-1 min-h-32 w-full resize-y rounded-md border border-slate-300 bg-white px-3 py-2 text-sm leading-6 text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                placeholder="Step-by-step behavior, constraints, and verification"
                :value="ctrl.draft.value.guidance"
                @input="ctrl.updateDraft('guidance', ($event.target as HTMLTextAreaElement).value)"
              />
            </label>
          </Fieldset>

          <Fieldset class="mt-4 rounded-lg border border-slate-200 bg-white p-4 shadow-md fieldset-nested" legend="Catalog Metrics" toggleable :collapsed="true">
            <div class="grid gap-3 md:grid-cols-3">
              <label class="block min-w-0">
                <span class="text-xs font-bold text-slate-500">Downloads</span>
                <input
                  class="mt-1 h-10 w-full rounded-md border border-slate-300 bg-white px-3 text-sm text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                  inputmode="numeric"
                  :value="String(ctrl.draft.value.downloads)"
                  @input="ctrl.updateDraft('downloads', Math.max(0, Number(($event.target as HTMLInputElement).value) || 0))"
                />
              </label>
              <label class="block min-w-0">
                <span class="text-xs font-bold text-slate-500">Stars</span>
                <input
                  class="mt-1 h-10 w-full rounded-md border border-slate-300 bg-white px-3 text-sm text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                  inputmode="numeric"
                  :value="String(ctrl.draft.value.stars)"
                  @input="ctrl.updateDraft('stars', Math.max(0, Number(($event.target as HTMLInputElement).value) || 0))"
                />
              </label>
              <label class="block min-w-0">
                <span class="text-xs font-bold text-slate-500">Installs</span>
                <input
                  class="mt-1 h-10 w-full rounded-md border border-slate-300 bg-white px-3 text-sm text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                  inputmode="numeric"
                  :value="String(ctrl.draft.value.installs)"
                  @input="ctrl.updateDraft('installs', Math.max(0, Number(($event.target as HTMLInputElement).value) || 0))"
                />
              </label>
            </div>
          </Fieldset>

          <!-- Generated Markdown -->
          <section class="mt-4 flex min-h-0 flex-col overflow-hidden rounded-lg border border-stone-200 bg-white">
            <div class="flex items-center justify-between gap-3 border-b border-stone-200 px-4 py-3">
              <h3 class="font-bold">Generated Markdown</h3>
              <button
                class="flex h-9 items-center gap-2 rounded-md border border-slate-300 bg-white px-3 text-sm font-bold text-slate-600 hover:bg-slate-50"
                type="button"
                title="Copy generated markdown"
                @click="copyMarkdown()"
              >
                <i class="pi pi-copy" />
                Copy
              </button>
            </div>
            <p v-if="copyMessage" class="border-b border-stone-200 px-4 py-2 text-xs text-slate-500">{{ copyMessage }}</p>
            <pre class="max-h-72 overflow-auto whitespace-pre-wrap p-4 font-mono text-xs leading-5 text-slate-800">{{ ctrl.generatedMarkdown.value }}</pre>
          </section>

          <div class="mt-4 flex justify-end">
            <button
              class="flex h-9 items-center gap-2 rounded-md border border-red-200 bg-white px-3 text-sm font-bold text-red-700 hover:bg-red-50"
              type="button"
              title="Delete skill"
              @click="confirmDelete"
            >
              <i class="pi pi-trash" />
              Delete
            </button>
          </div>
        </div>
      </section>
    </section>
  </section>
</template>
