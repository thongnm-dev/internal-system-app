<script setup lang="ts">
import { computed, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import Checkbox from "primevue/checkbox";
import { useAuthStore } from "@/app/stores/auth";
import {
  emptyProjectTaskInput,
  PROJECT_TASK_CATEGORIES,
  useProjectTasks,
  type ProjectTaskCategory,
} from "../composables/useProjectTasks";

const route = useRoute();
const router = useRouter();
const auth = useAuthStore();
const projectCode = (route.params.id as string) || "";

const ctrl = useProjectTasks(projectCode);
const form = ref(emptyProjectTaskInput(auth.user?.username ?? ""));

const canSave = computed(() => form.value.shortName.trim().length > 0);

function toggleCategory(category: ProjectTaskCategory) {
  const list = form.value.categories;
  form.value.categories = list.includes(category)
    ? list.filter((c) => c !== category)
    : [...list, category];
}

function goBack() {
  router.push(`/projects/${encodeURIComponent(projectCode)}/tasks`);
}

function saveTask() {
  if (!canSave.value) return;
  ctrl.addTask(form.value);
  goBack();
}
</script>

<template>
  <section class="min-h-0 flex-1 overflow-auto rounded-lg border border-divider bg-panel p-4 shadow-sm">
    <div class="flex items-center justify-between gap-4 border-b border-divider pb-3">
      <div class="min-w-0">
        <h3 class="truncate font-bold text-ink">New Task</h3>
        <p class="mt-1 truncate text-sm text-muted">
          {{ projectCode }}<template v-if="ctrl.projectName.value"> - {{ ctrl.projectName.value }}</template>
        </p>
      </div>
      <button
        class="flex h-9 shrink-0 items-center gap-2 rounded-md border border-divider bg-panel px-3 text-sm font-bold text-secondary hover:bg-canvas"
        type="button"
        @click="goBack"
      >
        <i class="pi pi-arrow-left" />Back
      </button>
    </div>

    <div class="mx-auto mt-4 grid max-w-2xl gap-4">
      <!-- Short name -->
      <label class="block">
        <span class="text-xs font-bold text-muted">Short name <span class="text-red-500">*</span></span>
        <input
          v-model="form.shortName"
          class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
          placeholder="Task short name"
          autofocus
        />
      </label>

      <!-- Description -->
      <label class="block">
        <span class="text-xs font-bold text-muted">Mô tả</span>
        <textarea
          v-model="form.description"
          class="mt-1 min-h-28 w-full resize-none rounded-md border border-divider bg-panel px-3 py-2 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
          placeholder="Task description"
        />
      </label>

      <!-- Task category (multi-select) -->
      <div>
        <span class="text-xs font-bold text-muted">Phân loại task</span>
        <div class="mt-1 flex flex-wrap gap-2">
          <label
            v-for="category in PROJECT_TASK_CATEGORIES"
            :key="category"
            :class="[
              'flex cursor-pointer items-center gap-2 rounded-md border px-3 py-2 text-sm font-semibold transition',
              form.categories.includes(category)
                ? 'border-brand bg-emerald-50 text-brand'
                : 'border-divider bg-panel text-secondary hover:border-brand',
            ]"
          >
            <Checkbox
              :model-value="form.categories.includes(category)"
              :binary="true"
              class="h-4 w-4 accent-brand"
              @update:model-value="toggleCategory(category)"
            />
            {{ category }}
          </label>
        </div>
      </div>

      <div class="grid gap-4 md:grid-cols-2">
        <!-- Assignee -->
        <label class="block">
          <span class="text-xs font-bold text-muted">Assignee</span>
          <input
            v-model="form.assignee"
            class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
            placeholder="Username"
          />
        </label>

        <!-- Estimate hour -->
        <label class="block">
          <span class="text-xs font-bold text-muted">Estimate Hour</span>
          <input
            v-model="form.estimateHour"
            class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
            inputmode="decimal"
            min="0"
            step="0.25"
            type="number"
            placeholder="0"
          />
        </label>

        <!-- Due date -->
        <label class="block">
          <span class="text-xs font-bold text-muted">Due Date</span>
          <input
            v-model="form.dueDate"
            class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
            type="date"
          />
        </label>

        <!-- Backlog issue key -->
        <label class="block">
          <span class="text-xs font-bold text-muted">Link Issue Backlog</span>
          <input
            v-model="form.issueKey"
            class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
            placeholder="Issue Key"
          />
        </label>
      </div>

      <div class="flex items-center justify-end gap-2 border-t border-divider pt-4">
        <button
          class="h-10 rounded-md border border-divider bg-panel px-4 text-sm font-bold text-secondary hover:bg-canvas"
          type="button"
          @click="goBack"
        >
          Cancel
        </button>
        <button
          class="h-10 rounded-md bg-brand px-4 text-sm font-bold text-white hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-50"
          type="button"
          :disabled="!canSave"
          @click="saveTask"
        >
          Add task
        </button>
      </div>
    </div>
  </section>
</template>
