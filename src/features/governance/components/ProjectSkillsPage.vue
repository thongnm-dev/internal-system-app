<script setup lang="ts">
import { computed, ref } from "vue";
import Button from "primevue/button";
import Dialog from "primevue/dialog";
import Fieldset from "primevue/fieldset";
import InputText from "primevue/inputtext";
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
const dialogVisible = ref(false);

const dialogHeader = computed(() =>
  ctrl.skills.value.some((s) => s.id === ctrl.draft.value.id)
    ? ctrl.draft.value.name || "Untitled Skill"
    : "New Skill",
);

function openSkill(skillId: string) {
  ctrl.selectSkill(skillId);
  copyMessage.value = "";
  dialogVisible.value = true;
}

function openNewSkill() {
  ctrl.createSkill();
  copyMessage.value = "";
  dialogVisible.value = true;
}

function saveSkill() {
  if (ctrl.saveDraft()) dialogVisible.value = false;
}

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
    dialogVisible.value = false;
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
    <section class="flex flex-wrap items-end gap-3 rounded-lg border border-divider bg-panel p-4 shadow-sm">
      <label class="block min-w-0 flex-1">
        <span class="text-xs font-bold text-muted">Search Skills</span>
        <span class="mt-1 flex h-10 items-center gap-2 rounded-md border border-divider bg-panel px-3 focus-within:border-brand focus-within:ring-2 focus-within:ring-emerald-100">
          <i class="pi pi-search shrink-0 text-muted" />
          <InputText
            class="w-full min-w-0 flex-1 !border-0 !bg-transparent !p-0 text-sm text-ink !shadow-none !outline-none !ring-0 focus:!border-0 focus:!bg-transparent focus:!shadow-none focus:!outline-none focus:!ring-0"
            placeholder="Name, tag, category, guidance"
            :model-value="ctrl.query.value"
            @update:model-value="ctrl.query.value = $event as string"
          />
        </span>
      </label>

      <Button icon="pi pi-plus" label="New" title="Create skill" class="shrink-0" @click="openNewSkill()" />
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

    <!-- Main: skill list -->
    <section class="flex min-h-0 flex-1 overflow-hidden">
      <!-- Skill list -->
      <section class="min-h-0 w-full flex-1 overflow-auto rounded-lg border border-divider bg-panel p-3 shadow-sm">
        <p v-if="ctrl.filteredSkills.value.length === 0" class="p-3 text-sm text-muted">No skills match the current filters.</p>
        <div v-else :class="ctrl.viewMode.value === 'grid' ? 'grid gap-3 sm:grid-cols-2 lg:grid-cols-3 2xl:grid-cols-4' : 'grid gap-2'">
          <Button
            v-for="skill in ctrl.filteredSkills.value"
            :key="skill.id"
            :class="skillCardClass(skill, ctrl.selectedSkillId.value === skill.id, ctrl.viewMode.value)"
            unstyled
            @click="openSkill(skill.id)"
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
    </section>

    <!-- Detail editor dialog -->
    <Dialog
      v-model:visible="dialogVisible"
      :header="dialogHeader"
      modal
      maximizable
      :style="{ width: '820px' }"
      :content-style="{ padding: '0' }"
    >
      <div class="flex flex-col gap-4 p-4">
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

          <Fieldset class="rounded-lg border border-divider bg-panel p-4 shadow-md fieldset-nested" legend="Guidance" toggleable>
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

          <!-- Generated Markdown -->
          <section class="flex min-h-0 flex-col overflow-hidden rounded-lg border border-divider bg-panel">
            <div class="flex items-center justify-between gap-3 border-b border-divider px-4 py-3">
              <h3 class="font-bold">Generated Markdown</h3>
              <Button icon="pi pi-copy" label="Copy" severity="secondary" outlined size="small" title="Copy generated markdown" @click="copyMarkdown()" />
            </div>
            <p v-if="copyMessage" class="border-b border-divider px-4 py-2 text-xs text-muted">{{ copyMessage }}</p>
            <pre class="max-h-72 overflow-auto whitespace-pre-wrap p-4 font-mono text-xs leading-5 text-ink">{{ ctrl.generatedMarkdown.value }}</pre>
          </section>
      </div>

      <template #footer>
        <div class="flex w-full items-center justify-between gap-2">
          <Button icon="pi pi-trash" label="Delete" severity="danger" outlined size="small" title="Delete skill" @click="confirmDelete" />
          <div class="flex items-center gap-2">
            <Button icon="pi pi-refresh" label="Reset" severity="secondary" outlined size="small" title="Reset draft" @click="ctrl.resetDraft()" />
            <Button icon="pi pi-save" label="Save" size="small" title="Save skill" @click="saveSkill()" />
          </div>
        </div>
      </template>
    </Dialog>
  </section>
</template>
