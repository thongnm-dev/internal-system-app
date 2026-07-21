<script setup lang="ts">
import { ref, computed } from "vue";
import { useRoute } from "vue-router";
import Button from "primevue/button";
import DataTable from "primevue/datatable";
import Column from "primevue/column";
import Fieldset from "primevue/fieldset";
import Calendar from "primevue/calendar";
import InputText from "primevue/inputtext";
import BacklogConfigError from "./BacklogConfigError.vue";
import ImportIssuesDialog from "./ImportIssuesPage.vue";
import {
  assignees,
  priorityOptions,
  categories,
  issueTypes,
  projects,
  statusOptions,
  uniqueCreateUsers,
  useIssueBacklog,
  statusTone,
  priorityTone,
} from "../composables/useIssueBacklog";
import { canUseTauriRuntime } from "@/tauri/commands/_base";
import { backlogGetBaseUrl } from "@/tauri/commands/backlog";
import { useToast } from "@/shared/composables/useToast";

const route = useRoute();
const toast = useToast();
const initialProject = (route.query.project as string) || "";
const ctrl = useIssueBacklog(initialProject);

// Khi chưa chọn dự án (hoặc đang tải lookup) thì khóa toàn bộ điều kiện tìm kiếm.
const searchDisabled = computed(() => !ctrl.criteria.value.project || ctrl.lookupLoading.value);

// --- Mở issue Backlog trong cửa sổ webview để cập nhật trực tiếp ---
let cachedBaseUrl = "";
const openingIssue = ref("");

async function openIssueWebview(issueKey: string) {
  if (!issueKey || !canUseTauriRuntime()) return;
  openingIssue.value = issueKey;
  try {
    if (!cachedBaseUrl) {
      cachedBaseUrl = (await backlogGetBaseUrl()).replace(/\/+$/, "");
    }
    const url = `${cachedBaseUrl}/view/${encodeURIComponent(issueKey)}`;
    const label = `backlog-issue-${issueKey.replace(/[^a-zA-Z0-9_-]/g, "-")}`;

    const { WebviewWindow } = await import("@tauri-apps/api/webviewWindow");
    const existing = await WebviewWindow.getByLabel(label);
    if (existing) {
      await existing.setFocus();
      return;
    }

    const win = new WebviewWindow(label, {
      url,
      title: `Backlog - ${issueKey}`,
      width: 1600,
      height: 820,
      center: true,
      resizable: true,
    });
    win.once("tauri://error", (event) => {
      toast.error(`Failed to open issue: ${String(event.payload)}`);
    });
  } catch (e) {
    toast.error(`Failed to open issue: ${String(e)}`);
  } finally {
    openingIssue.value = "";
  }
}

function toggleStatus(s: string) {
  ctrl.setField("notClosed", false);
  const current = ctrl.criteria.value.status;
  const next = current.includes(s) ? current.filter((v) => v !== s) : [...current, s];
  ctrl.setField("status", next);
}

function selectAll() {
  ctrl.setField("notClosed", false);
  ctrl.setField("status", []);
}

function selectNotClosed() {
  ctrl.setField("status", []);
  ctrl.setField("notClosed", true);
}

// --- Import to Backlog dialog ---
const importDialogVisible = ref(false);
</script>

<template>
  <BacklogConfigError
    v-if="ctrl.configError.value"
    :is-checking="ctrl.configChecking.value"
    @retry="ctrl.checkConfig()"
  />

  <section v-else class="flex min-h-0 flex-1 flex-col gap-1 overflow-x-hidden overflow-y-auto">
    <Fieldset
      class="rounded-lg border border-divider bg-panel p-4 shadow-md fieldset-nested"
      legend="Search"
      toggleable
    >
      <div class="grid gap-3">
        <div class="grid items-end gap-3 lg:grid-cols-2">
          <label class="block min-w-0">
            <span class="text-xs font-bold text-muted">Project</span>
            <select
              class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
              :value="ctrl.criteria.value.project"
              @change="ctrl.setField('project', ($event.target as HTMLSelectElement).value)"
            >
              <option value="">All projects</option>
              <option v-for="p in projects" :key="p.code" :value="p.code">{{ p.name }}</option>
            </select>
          </label>

          <div class="block min-w-0">
            <span class="text-xs font-bold text-muted">Status</span>
            <div class="mt-1 flex h-10 min-w-0 flex-wrap items-center gap-1 rounded-md border border-divider bg-panel p-1 text-sm leading-none">
              <Button
                :class="[
                  'flex min-w-0 items-center justify-center truncate rounded px-2 py-1 text-sm font-normal transition',
                  ctrl.criteria.value.status.length === 0 && !ctrl.criteria.value.notClosed ? 'bg-brand text-white' : 'hover:bg-canvas',
                ]"
                unstyled
                :disabled="searchDisabled"
                @click="selectAll()"
              >
                All
              </Button>
              <Button
                v-for="s in statusOptions"
                :key="s"
                :class="[
                  'flex min-w-0 items-center justify-center truncate rounded px-2 py-1 text-sm font-normal transition',
                  !ctrl.criteria.value.notClosed && ctrl.criteria.value.status.includes(s) ? 'bg-brand text-white' : 'hover:bg-canvas',
                ]"
                unstyled
                :disabled="searchDisabled"
                @click="toggleStatus(s)"
              >
                {{ s }}
              </Button>
              <Button
                :class="[
                  'flex min-w-0 items-center justify-center truncate rounded px-2 py-1 text-sm font-normal transition',
                  ctrl.criteria.value.notClosed ? 'bg-brand text-white' : 'hover:bg-canvas',
                ]"
                unstyled
                :disabled="searchDisabled || statusOptions.length === 0"
                @click="selectNotClosed()"
              >
                Not Closed
              </Button>
            </div>
          </div>
        </div>

        <div class="grid gap-3 lg:grid-cols-2">
          <label class="block min-w-0">
            <span class="text-xs font-bold text-muted">Issue Type</span>
            <select
              class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
              :disabled="searchDisabled"
              :value="ctrl.criteria.value.issueType"
              @change="ctrl.setField('issueType', ($event.target as HTMLSelectElement).value)"
            >
              <option value="">All types</option>
              <option v-for="t in issueTypes" :key="t" :value="t">{{ t }}</option>
            </select>
          </label>
          <div class="grid gap-3 md:grid-cols-2">
            <label class="block min-w-0">
              <span class="text-xs font-bold text-muted">Category</span>
              <select
                class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                :disabled="searchDisabled"
                :value="ctrl.criteria.value.category"
                @change="ctrl.setField('category', ($event.target as HTMLSelectElement).value)"
              >
                <option value="">All categories</option>
                <option v-for="c in categories" :key="c" :value="c">{{ c }}</option>
              </select>
            </label>
            <label class="block min-w-0">
              <span class="text-xs font-bold text-muted">Assignee</span>
              <select
                class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                :disabled="searchDisabled"
                :value="ctrl.criteria.value.assignee"
                @change="ctrl.setField('assignee', ($event.target as HTMLSelectElement).value)"
              >
                <option value="">All assignees</option>
                <option v-for="a in assignees" :key="a" :value="a">{{ a }}</option>
              </select>
            </label>
          </div>
        </div>

        <div class="grid gap-3 lg:grid-cols-2">
          <label class="block min-w-0">
            <span class="text-xs font-bold text-muted">Keyword</span>
            <InputText
              class="mt-1 w-full"
              placeholder="Issue key or subject"
              :disabled="searchDisabled"
              :model-value="ctrl.criteria.value.keyword"
              @update:model-value="ctrl.setField('keyword', $event as string)"
              @keydown.enter="ctrl.search()"
            />
          </label>
          <div />
        </div>

        <Fieldset
          class="rounded-md border border-divider bg-panel p-3 fieldset-nested"
          legend="Advanced"
          toggleable
        >
          <div class="grid gap-3 lg:grid-cols-2">
            <div class="grid gap-3 md:grid-cols-2">
              <label class="block min-w-0">
                <span class="text-xs font-bold text-muted">Create date from</span>
                <Calendar
                  :model-value="ctrl.criteria.value.createDateFrom ? new Date(ctrl.criteria.value.createDateFrom + 'T00:00:00') : null"
                  :disabled="searchDisabled"
                  class="mt-1 w-full"
                  date-format="yy/mm/dd"
                  placeholder="Select date"
                  show-icon
                  show-button-bar
                  @update:model-value="ctrl.setField('createDateFrom', $event ? `${$event.getFullYear()}-${String($event.getMonth() + 1).padStart(2, '0')}-${String($event.getDate()).padStart(2, '0')}` : '')"
                />
              </label>
              <label class="block min-w-0">
                <span class="text-xs font-bold text-muted">Create date to</span>
                <Calendar
                  :model-value="ctrl.criteria.value.createDateTo ? new Date(ctrl.criteria.value.createDateTo + 'T00:00:00') : null"
                  :disabled="searchDisabled"
                  class="mt-1 w-full"
                  date-format="yy/mm/dd"
                  placeholder="Select date"
                  show-icon
                  show-button-bar
                  @update:model-value="ctrl.setField('createDateTo', $event ? `${$event.getFullYear()}-${String($event.getMonth() + 1).padStart(2, '0')}-${String($event.getDate()).padStart(2, '0')}` : '')"
                />
              </label>
            </div>
            <div class="grid gap-3 md:grid-cols-2">
              <label class="block min-w-0">
                <span class="text-xs font-bold text-muted">Create user</span>
                <select
                  class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                  :disabled="searchDisabled"
                  :value="ctrl.criteria.value.createUser"
                  @change="ctrl.setField('createUser', ($event.target as HTMLSelectElement).value)"
                >
                  <option value="">All users</option>
                  <option v-for="u in uniqueCreateUsers" :key="u" :value="u">{{ u }}</option>
                </select>
              </label>
              <label class="block min-w-0">
                <span class="text-xs font-bold text-muted">Priority</span>
                <select
                  class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                  :disabled="searchDisabled"
                  :value="ctrl.criteria.value.priorityFilter"
                  @change="ctrl.setField('priorityFilter', ($event.target as HTMLSelectElement).value)"
                >
                  <option value="">All priorities</option>
                  <option v-for="p in priorityOptions" :key="p" :value="p">{{ p }}</option>
                </select>
              </label>
            </div>
          </div>
        </Fieldset>

        <div class="flex items-center justify-end gap-2">
          <Button icon="pi pi-refresh" label="Reset" severity="secondary" outlined title="Reset search conditions" @click="ctrl.reset()" />
          <Button
            icon="pi pi-file-import"
            label="Import"
            :disabled="!ctrl.canOpenImport.value"
            :title="ctrl.canOpenImport.value ? 'Import issues to Backlog' : 'Select a project before importing'"
            @click="importDialogVisible = true"
          />
          <Button icon="pi pi-search" label="Search" :disabled="!ctrl.criteria.value.project" title="Search" @click="ctrl.search()" />
        </div>
      </div>
    </Fieldset>

    <div v-if="ctrl.searchError.value" class="rounded-md border border-red-300 bg-red-50 p-3 text-sm text-red-700">
      {{ ctrl.searchError.value }}
    </div>

    <section class="flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border border-divider bg-panel shadow-sm">
      <div class="flex items-center justify-between gap-4 border-b border-divider px-4 py-3">
        <h3 class="font-bold">Issue backlog list</h3>
        <span class="text-xs text-muted">{{ ctrl.totalCount.value.toLocaleString("en-US") }} issues</span>
      </div>

      <DataTable
        class="app-data-table min-h-0"
        empty-message="No issues match the search conditions."
        scrollable
        scroll-height="flex"
        :table-style="{ minWidth: '1180px' }"
        :value="ctrl.filteredItems.value"
        lazy
        paginator
        :first="ctrl.first.value"
        :rows="ctrl.pageSize.value"
        :total-records="ctrl.totalCount.value"
        :rows-per-page-options="[20, 50, 100]"
        paginator-template="FirstPageLink PrevPageLink PageLinks NextPageLink LastPageLink RowsPerPageDropdown CurrentPageReport"
        current-page-report-template="Showing {first} to {last} of {totalRecords}"
        @page="ctrl.onPage($event)"
      >
        <Column field="issueType" header="Issue Type" body-class="whitespace-nowrap" />
        <Column field="issueKey" header="Issue Key" body-class="whitespace-nowrap font-bold text-ink" />
        <Column field="subject" header="Subject" body-class="max-w-[360px] truncate" />
        <Column field="assignee" header="Assignee" body-class="whitespace-nowrap" />
        <Column header="Status" body-class="whitespace-nowrap">
          <template #body="{ data }">
            <span :class="['rounded px-2 py-1 text-xs font-bold', statusTone(data.status)]">{{ data.status }}</span>
          </template>
        </Column>
        <Column header="Hours" body-class="num" header-class="num">
          <template #body="{ data }">{{ data.hours.toFixed(data.hours % 1 === 0 ? 0 : 1) }}</template>
        </Column>
        <Column header="Priority" body-class="whitespace-nowrap">
          <template #body="{ data }">
            <span :class="['rounded px-2 py-1 text-xs font-bold', priorityTone(data.priority)]">{{ data.priority }}</span>
          </template>
        </Column>
        <Column field="createDate" header="Create Date" body-class="whitespace-nowrap" />
        <Column field="createUser" header="Create User" body-class="whitespace-nowrap" />
        <Column header="" header-class="w-28" body-class="whitespace-nowrap text-right">
          <template #body="{ data }">
            <Button
              icon="pi pi-external-link"
              label="View"
              size="small"
              outlined
              :loading="openingIssue === data.issueKey"
              title="Open the issue in Backlog to update it"
              @click="openIssueWebview(data.issueKey)"
            />
          </template>
        </Column>
      </DataTable>
    </section>

    <ImportIssuesDialog
      v-model:visible="importDialogVisible"
      :project-code="ctrl.criteria.value.project"
    />
  </section>
</template>

