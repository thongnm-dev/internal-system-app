<script setup lang="ts">
import { computed, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import Button from "primevue/button";
import DataTable from "primevue/datatable";
import Column from "primevue/column";
import Dialog from "primevue/dialog";
import MultiSelect from "primevue/multiselect";
import Calendar from "primevue/calendar";
import InputNumber from "primevue/inputnumber";
import InputText from "primevue/inputtext";
import { useAuthStore } from "@/app/stores/auth";
import { friendlyError } from "@/tauri/commands/_base";
import {
  emptyProjectTaskInput,
  useProjectTasks,
  type ProjectTask,
} from "../composables/useProjectTasks";

const route = useRoute();
const router = useRouter();
const auth = useAuthStore();
const projectId = (route.params.id as string) || "";

const ctrl = useProjectTasks(projectId);

// Chỉ cho phép liên kết Issue Backlog khi dự án đã thiết lập Backlog key.
const hasBacklog = computed(() => Boolean(ctrl.project.value?.backlog_key));

const categoryNameMap = computed(() => new Map(ctrl.categories.value.map((c) => [c.code, c.name])));
function categoryName(code: string) {
  return categoryNameMap.value.get(code) ?? code;
}

const isAddDialogOpen = ref(false);
const editingId = ref<string | null>(null);
const form = ref(emptyProjectTaskInput(auth.user?.username ?? ""));
const saving = ref(false);
const saveError = ref("");
const canSave = computed(() => form.value.shortName.trim().length > 0);
const isEditing = computed(() => editingId.value !== null);

function importTask() {
  router.push(`/projects/${encodeURIComponent(projectId)}/tasks/new`);
}

function viewReport() {
  router.push(`/projects/${encodeURIComponent(projectId)}/report`);
}

function openBacklog(issueKey: string) {
  if (!issueKey) return;
  router.push({ path: "/issue-backlog", query: { keyword: issueKey } });
}

function openAddDialog() {
  editingId.value = null;
  form.value = emptyProjectTaskInput(auth.user?.username ?? "");
  saveError.value = "";
  isAddDialogOpen.value = true;
}

function openEditDialog(task: ProjectTask) {
  editingId.value = task.id;
  form.value = {
    shortName: task.shortName,
    description: task.description,
    categories: [...task.categories],
    assignee: task.assignee,
    estimateHour: task.estimateHour,
    dueDate: task.dueDate,
    issueKey: task.issueKey,
  };
  saveError.value = "";
  isAddDialogOpen.value = true;
}

async function saveTask() {
  if (!canSave.value || saving.value) return;
  saving.value = true;
  saveError.value = "";
  try {
    const task = editingId.value
      ? await ctrl.updateTask(editingId.value, form.value)
      : await ctrl.addTask(form.value);
    if (!task) {
      saveError.value = "Cannot save task. The desktop runtime is unavailable.";
      return;
    }
    isAddDialogOpen.value = false;
  } catch (error) {
    saveError.value = friendlyError(error);
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
    <section class="flex items-center justify-between gap-4 rounded-lg border border-divider bg-panel p-4 shadow-sm">
      <div class="min-w-0">
        <h3 class="truncate font-bold text-ink">Tasks</h3>
        <p class="mt-1 truncate text-sm text-muted">{{ ctrl.projectLabel.value }}</p>
      </div>
      <div class="flex shrink-0 items-center gap-2">
        <Button icon="pi pi-chart-bar" label="Report" severity="secondary" outlined @click="viewReport" />
        <Button icon="pi pi-upload" label="Import task" severity="secondary" outlined @click="importTask" />
        <Button icon="pi pi-plus" label="Add task" @click="openAddDialog" />
      </div>
    </section>

    <section class="flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border border-divider bg-panel shadow-sm">
      <div class="flex items-center justify-between gap-4 border-b border-divider px-4 py-3">
        <h3 class="font-bold">Task list</h3>
        <span class="text-xs text-muted">{{ ctrl.tasks.value.length.toLocaleString("en-US") }} tasks</span>
      </div>
      <DataTable
        class="app-data-table min-h-0"
        empty-message="No tasks yet. Click Add task to create one."
        scrollable
        scroll-height="flex"
        :table-style="{ minWidth: '980px' }"
        :value="ctrl.tasks.value"
        paginator
        :rows="20"
        :rows-per-page-options="[20, 50, 100]"
        paginator-template="FirstPageLink PrevPageLink PageLinks NextPageLink LastPageLink RowsPerPageDropdown CurrentPageReport"
        current-page-report-template="Showing {first} to {last} of {totalRecords}"
      >
        <Column field="shortName" header="Short name" body-class="font-bold text-ink" />
        <Column header="Category">
          <template #body="{ data }">
            <div class="flex flex-wrap gap-1">
              <span
                v-for="category in data.categories"
                :key="category"
                class="rounded-md bg-emerald-50 px-2 py-0.5 text-xs font-bold text-brand"
              >
                {{ categoryName(category) }}
              </span>
              <span v-if="!data.categories.length" class="text-muted">-</span>
            </div>
          </template>
        </Column>
        <Column field="assignee" header="Assignee">
          <template #body="{ data }">{{ data.assignee || '-' }}</template>
        </Column>
        <Column header="Estimate" body-class="num" header-class="num">
          <template #body="{ data }">{{ data.estimateHour ? `${data.estimateHour}h` : '-' }}</template>
        </Column>
        <Column field="dueDate" header="Due date">
          <template #body="{ data }">{{ data.dueDate || '-' }}</template>
        </Column>
        <Column header="Issue">
          <template #body="{ data }">
            <Button
              v-if="data.issueKey"
              :label="data.issueKey"
              text
              size="small"
              class="font-bold text-brand"
              @click="openBacklog(data.issueKey)"
            />
            <span v-else class="text-muted">-</span>
          </template>
        </Column>
        <Column header="Action" body-class="text-center" header-class="w-28 text-center">
          <template #body="{ data }">
            <div class="flex items-center justify-center gap-1">
              <Button icon="pi pi-pencil" severity="secondary" outlined size="small" title="Edit task" @click="openEditDialog(data)" />
              <Button icon="pi pi-trash" severity="secondary" outlined size="small" title="Delete task" @click="ctrl.removeTask(data.id)" />
            </div>
          </template>
        </Column>
      </DataTable>
    </section>

    <!-- Add task dialog -->
    <Dialog
      :visible="isAddDialogOpen"
      class="w-full max-w-2xl rounded-lg bg-panel shadow-xl"
      :closable="true"
      modal
      @update:visible="isAddDialogOpen = $event"
    >
      <template #header>
        <div>
          <h3 class="font-bold text-ink">{{ isEditing ? "Edit Task" : "New Task" }}</h3>
          <p class="mt-1 text-sm text-muted">{{ ctrl.projectLabel.value }}</p>
        </div>
      </template>

      <div class="space-y-4">
        <label class="block">
          <span class="text-xs font-bold text-muted">Short name <span class="text-red-500">*</span></span>
          <InputText
            v-model="form.shortName"
            class="mt-1 w-full"
            placeholder="Task short name"
            autofocus
          />
        </label>

        <label class="block">
          <span class="text-xs font-bold text-muted">Description</span>
          <textarea
            v-model="form.description"
            class="mt-1 min-h-20 w-full resize-none rounded-md border border-divider bg-panel px-3 py-2 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
            placeholder="Task description"
          />
        </label>

        <label class="block">
          <span class="text-xs font-bold text-muted">Category</span>
          <MultiSelect
            v-model="form.categories"
            :options="ctrl.categories.value"
            option-label="name"
            option-value="code"
            placeholder="Select categories"
            display="chip"
            class="mt-1 w-full"
          />
        </label>

        <div class="grid gap-4 md:grid-cols-2">
          <label class="block">
            <span class="text-xs font-bold text-muted">Assignee</span>
            <InputText
              v-model="form.assignee"
              class="mt-1 w-full"
              placeholder="Username"
            />
          </label>
          <label class="block">
            <span class="text-xs font-bold text-muted">Estimate Hour</span>
            <InputNumber
              :model-value="Number(form.estimateHour) || null"
              class="mt-1 w-full"
              :min="0"
              :step="0.25"
              :minFractionDigits="0"
              :maxFractionDigits="2"
              :useGrouping="false"
              placeholder="0"
              @update:model-value="form.estimateHour = String($event ?? '')"
            />
          </label>
          <label class="block">
            <span class="text-xs font-bold text-muted">Due Date</span>
            <Calendar
              :model-value="form.dueDate ? new Date(form.dueDate + 'T00:00:00') : null"
              class="mt-1 w-full"
              date-format="yy/mm/dd"
              placeholder="Select date"
              show-icon
              show-button-bar
              @update:model-value="form.dueDate = $event ? `${$event.getFullYear()}-${String($event.getMonth() + 1).padStart(2, '0')}-${String($event.getDate()).padStart(2, '0')}` : ''"
            />
          </label>
          <label class="block">
            <span class="text-xs font-bold text-muted">Link Issue Backlog</span>
            <InputText
              v-model="form.issueKey"
              class="mt-1 w-full"
              placeholder="Issue Key"
              :disabled="!hasBacklog || ctrl.projectLoading.value"
              :title="hasBacklog ? 'Link a Backlog issue key' : 'Project has no Backlog configured'"
            />
          </label>
        </div>
      </div>

      <template #footer>
        <div class="flex flex-col gap-2">
          <p v-if="saveError" class="text-right text-sm font-semibold text-red-500">{{ saveError }}</p>
          <div class="flex items-center justify-end gap-2">
            <Button label="Cancel" severity="secondary" outlined @click="isAddDialogOpen = false" />
            <Button
              :label="saving ? 'Saving…' : isEditing ? 'Save changes' : 'Add task'"
              :disabled="!canSave || saving"
              @click="saveTask"
            />
          </div>
        </div>
      </template>
    </Dialog>
  </section>
</template>
