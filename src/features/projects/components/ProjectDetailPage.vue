<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import DataTable from "primevue/datatable";
import Column from "primevue/column";
import Dialog from "primevue/dialog";
import Fieldset from "primevue/fieldset";
import Button from "primevue/button";
import InputText from "primevue/inputtext";
import IconActionButton from "@/shared/components/IconActionButton.vue";
import DialogFooter from "@/shared/components/DialogFooter.vue";
import { friendlyError } from "@/tauri/commands/_base";
import { createProject, updateProject, getBacklogProjectByKey, getProjectDetail } from "@/tauri/commands/project";
import { useToast } from "@/shared/composables/useToast";
import { useMembersStore } from "@/app/stores/members";
import { useDataTablePagination } from "@/shared/composables/useDataTablePagination";
import type { ProjectMember } from "@/_/types/project";

type ProjectForm = {
  id: number | null;
  code: string;
  name: string;
  client: string;
  backlogKey: string;
  backlogCode: string;
  backlogName: string;
};

const { paginationCompact } = useDataTablePagination();
const emptyForm: ProjectForm = { id: null, code: "", name: "", client: "", backlogKey: "", backlogCode: "", backlogName: "" };

/** Giới hạn ký tự khớp với độ dài cột trong bảng `projects` (docs/database/schema.sql). */
const MAX_LEN = { code: 20, name: 200, client: 200, backlogKey: 20, backlogName: 100 } as const;

const route = useRoute();
const router = useRouter();
const toast = useToast();
const membersStore = useMembersStore();
const projectID = route.params.id ? Number(route.params.id) : null;

const form = ref<ProjectForm>({ ...emptyForm });
const isLoading = ref(false);
const isSaving = ref(false);
const isBacklogLookupLoading = ref(false);
const isSearchHelpOpen = ref(false);
const backlogLookupError = ref("");
const loadError = ref("");
const saveError = ref("");
const memberKeyword = ref("");
const members = ref<ProjectMember[]>([]);
const fieldErrors = ref<Partial<Record<keyof ProjectForm, string>>>({});
const selectedMembers = ref<ProjectMember[]>([]);
const pendingMembers = ref<ProjectMember[]>([]);
const isConfirmStage = ref(false);

function normalize(v: string) { return v.trim().toLocaleLowerCase(); }

const filteredMembers = computed(() => {
  const kw = normalize(memberKeyword.value);
  const selected = new Set(members.value.map((m) => m.username));
  return membersStore.members.filter((m) => !selected.has(m.username) && (!kw || normalize(`${m.username} ${m.name}`).includes(kw)));
});

function openMemberSearch() {
  memberKeyword.value = "";
  selectedMembers.value = [];
  pendingMembers.value = [];
  isConfirmStage.value = false;
  isSearchHelpOpen.value = true;
  void membersStore.load();
}

/** Bấm "Select": chuyển các dòng đã tick sang bước xác nhận. */
function goToConfirm() {
  if (selectedMembers.value.length === 0) return;
  pendingMembers.value = [...selectedMembers.value].sort((a, b) => a.username.localeCompare(b.username));
  isConfirmStage.value = true;
}

/** Bấm "Confirm": thêm các member đã xác nhận vào project và đóng dialog. */
function confirmMembers() {
  const merged = new Map(members.value.map((m) => [m.username, m]));
  for (const m of pendingMembers.value) merged.set(m.username, m);
  members.value = [...merged.values()].sort((a, b) => a.username.localeCompare(b.username));
  isSearchHelpOpen.value = false;
}

function removeMember(username: string) {
  members.value = members.value.filter((m) => m.username !== username);
}

function updateForm(key: keyof ProjectForm, value: string) {
  form.value = { ...form.value, [key]: value };
  if (fieldErrors.value[key]) fieldErrors.value = { ...fieldErrors.value, [key]: undefined };
}

/** Kiểm tra các hạng mục bắt buộc/độ dài; trả về true nếu hợp lệ. */
function validateForm(): boolean {
  const errors: Partial<Record<keyof ProjectForm, string>> = {};
  const code = form.value.code.trim();
  const name = form.value.name.trim();

  if (!code) errors.code = "Project code is required.";
  else if (code.length > MAX_LEN.code) errors.code = `Project code must be at most ${MAX_LEN.code} characters.`;

  if (!name) errors.name = "Project name is required.";
  else if (name.length > MAX_LEN.name) errors.name = `Project name must be at most ${MAX_LEN.name} characters.`;

  if (form.value.client.length > MAX_LEN.client) errors.client = `Client must be at most ${MAX_LEN.client} characters.`;
  if (form.value.backlogKey.length > MAX_LEN.backlogKey) errors.backlogKey = `Backlog key must be at most ${MAX_LEN.backlogKey} characters.`;

  fieldErrors.value = errors;
  return Object.keys(errors).length === 0;
}

onMounted(async () => {
  if (!projectID) return;
  isLoading.value = true;
  try {
    const project = await getProjectDetail(projectID);
    form.value = {
      id: project.id,
      code: project.code,
      name: project.name,
      client: project.client,
      backlogKey: project.backlog_key,
      backlogCode: project.backlog_code,
      backlogName: project.backlog_name,
    };
    members.value = project.members;
  } catch (e) {
    form.value = { ...emptyForm };
    loadError.value = friendlyError(e);
  } finally {
    isLoading.value = false;
  }
});

async function saveProject() {
  saveError.value = "";
  if (!validateForm()) {
    toast.error("Please fix the highlighted fields before saving.");
    return;
  }
  isSaving.value = true;
  try {
    const request = {
      code: form.value.code,
      name: form.value.name,
      client: form.value.client || undefined,
      backlog_key: form.value.backlogKey || undefined,
      backlog_code: form.value.backlogCode || undefined,
      backlog_name: form.value.backlogName || undefined,
      members: members.value,
    };
    if (projectID) {
      const updated = await updateProject(projectID, request);
      form.value = { ...form.value, id: updated.id, code: updated.code, name: updated.name };
      toast.success("Project updated successfully.");
    } else {
      await createProject(request);
      form.value = { ...emptyForm };
      members.value = [];
      toast.success("Project created successfully.");
    }
  } catch (e) {
    saveError.value = friendlyError(e);
  } finally {
    isSaving.value = false;
  }
}

function reloadBacklogProject() {
  const trimmed = form.value.backlogKey.trim();
  if (!trimmed) {
    backlogLookupError.value = "";
    form.value = { ...form.value, backlogCode: "", backlogName: "" };
    return;
  }
  isBacklogLookupLoading.value = true;
  backlogLookupError.value = "";
  getBacklogProjectByKey(trimmed)
    .then((project) => {
      form.value = {
        ...form.value,
        backlogCode: String(project.projectId),
        backlogName: project.projectName,
      };
    })
    .catch((e) => {
      form.value = { ...form.value, backlogName: "", backlogCode: "" };
      backlogLookupError.value = friendlyError(e);
    })
    .finally(() => {
      isBacklogLookupLoading.value = false;
    });
}
</script>

<template>
  <section class="min-h-0 flex-1 overflow-auto rounded-lg border border-divider bg-panel p-4 shadow-sm">
    <div class="flex items-center justify-end gap-4">
      <Button :icon="isSaving ? 'pi pi-spinner pi-spin' : 'pi pi-save'" :label="isSaving ? 'Saving...' : 'Save'" size="small" :disabled="isSaving" @click="saveProject" />
    </div>

    <p v-if="isLoading" class="mt-4 text-sm text-muted">Loading project information...</p>
    <p v-if="loadError" class="banner-warning mt-4">{{ loadError }}</p>
    <p v-if="saveError" class="banner-danger mt-4">{{ saveError }}</p>

    <div class="mt-4 grid gap-4">
      <Fieldset class="rounded-lg border border-divider bg-panel p-4 shadow-md fieldset-nested" toggleable legend="Project Information">
        <div class="grid gap-3 md:grid-cols-2">
          <label>
            <span class="text-xs font-bold text-muted">Project ID</span>
            <InputText class="mt-1 w-full" placeholder="Auto generated" readonly :model-value="String(form.id ?? '')" />
          </label>
          <label>
            <span class="text-xs font-bold text-muted">Project Code <span class="text-red-500">*</span></span>
            <InputText
              class="mt-1 w-full"
              :class="{ 'p-invalid': fieldErrors.code }"
              placeholder="Project code"
              :maxlength="MAX_LEN.code"
              :model-value="form.code"
              @update:model-value="updateForm('code', $event as string)"
            />
            <small v-if="fieldErrors.code" class="mt-1 block text-xs text-red-500">{{ fieldErrors.code }}</small>
          </label>
          <label class="md:col-span-2">
            <span class="text-xs font-bold text-muted">Project Name <span class="text-red-500">*</span></span>
            <InputText
              class="mt-1 w-full"
              :class="{ 'p-invalid': fieldErrors.name }"
              placeholder="Project name"
              :maxlength="MAX_LEN.name"
              :model-value="form.name"
              @update:model-value="updateForm('name', $event as string)"
            />
            <small v-if="fieldErrors.name" class="mt-1 block text-xs text-red-500">{{ fieldErrors.name }}</small>
          </label>
          <label class="md:col-span-2">
            <span class="text-xs font-bold text-muted">Client</span>
            <InputText
              class="mt-1 w-full"
              :class="{ 'p-invalid': fieldErrors.client }"
              placeholder="Client name"
              :maxlength="MAX_LEN.client"
              :model-value="form.client"
              @update:model-value="updateForm('client', $event as string)"
            />
            <small v-if="fieldErrors.client" class="mt-1 block text-xs text-red-500">{{ fieldErrors.client }}</small>
          </label>
          <Fieldset class="rounded-lg border border-divider p-4 md:col-span-2 fieldset-nested" legend="Backlog" toggleable>
            <div class="grid gap-3 md:grid-cols-2">
              <div>
                <span class="text-xs font-bold text-muted">Backlog Key</span>
                <div class="group/key mt-1 flex rounded-md ring-brand/20 focus-within:ring-2">
                  <InputText
                    class="min-w-0 flex-1 !rounded-r-none"
                    :class="{ 'p-invalid': fieldErrors.backlogKey }"
                    placeholder="BACKLOG_KEY"
                    :maxlength="MAX_LEN.backlogKey"
                    :model-value="form.backlogKey"
                    @update:model-value="updateForm('backlogKey', $event as string)"
                  />
                  <Button
                    class="!rounded-l-none"
                    :icon="isBacklogLookupLoading ? 'pi pi-refresh pi-spin' : 'pi pi-refresh'"
                    severity="secondary"
                    outlined
                    title="Reload from Backlog API"
                    :disabled="isBacklogLookupLoading || !form.backlogKey.trim()"
                    @click="reloadBacklogProject"
                  />
                </div>
                <small v-if="fieldErrors.backlogKey" class="mt-1 block text-xs text-red-500">{{ fieldErrors.backlogKey }}</small>
              </div>
              <label>
                <span class="text-xs font-bold text-muted">Backlog Code</span>
                <InputText class="mt-1 w-full" disabled :placeholder="isBacklogLookupLoading ? 'Loading...' : ''" :model-value="form.backlogCode" />
              </label>
              <label class="md:col-span-2">
                <span class="text-xs font-bold text-muted">Backlog Name</span>
                <InputText class="mt-1 w-full" disabled :placeholder="isBacklogLookupLoading ? 'Loading...' : ''" :model-value="form.backlogName" />
              </label>
              <p v-if="backlogLookupError" class="text-sm text-red-500 md:col-span-2">{{ backlogLookupError }}</p>
            </div>
          </Fieldset>
        </div>
      </Fieldset>

      <Fieldset class="rounded-lg border border-divider bg-panel p-4 shadow-md fieldset-nested" legend="Members" toggleable>
        <div class="flex flex-wrap items-center justify-between gap-3">
          <h3 class="section-title">Members</h3>
          <Button icon="pi pi-plus" label="Add member" @click="openMemberSearch" />
        </div>
        <div class="mt-4 overflow-auto rounded-lg border border-divider">
          <DataTable class="app-data-table" empty-message="No members selected." :table-style="{ minWidth: '560px' }" :value="members">
            <Column field="username" header="Username" body-class="font-bold text-ink" />
            <Column field="name" header="Name" />
            <Column header="Action" body-class="text-center" header-class="w-20 text-center">
              <template #body="{ data }">
                <IconActionButton icon="pi pi-trash" severity="danger" title="Remove member" @click="removeMember(data.username)" />
              </template>
            </Column>
          </DataTable>
        </div>
      </Fieldset>
    </div>

    <!-- Search Help Dialog -->
    <Dialog
      :visible="isSearchHelpOpen"
      class="w-full max-w-2xl rounded-lg bg-panel shadow-xl"
      :closable="true"
      modal
      @update:visible="isSearchHelpOpen = $event"
    >
      <template #header>
        <div>
          <h3 class="section-title">{{ isConfirmStage ? "Confirm members" : "Search help members" }}</h3>
          <p class="mt-1 text-sm text-muted">
            {{ isConfirmStage ? "Review the selected members before adding them to the project." : "Search and select members to add to the project." }}
          </p>
        </div>
      </template>

      <!-- Stage 1: search + pick rows with checkbox -->
      <div v-if="!isConfirmStage" class="grid gap-4">
        <Fieldset class="rounded-lg border border-divider fieldset-nested" legend="Search" toggleable>
          <label class="block">
            <span class="text-xs font-bold text-muted">Keyword</span>
            <span class="mt-1 flex items-stretch gap-2">
              <InputText class="min-w-0 flex-1" placeholder="Search username or name" v-model="memberKeyword" />
              <Button
                icon="pi pi-times"
                severity="secondary"
                outlined
                title="Clear keyword"
                :disabled="!memberKeyword"
                @click="memberKeyword = ''"
              />
            </span>
          </label>
        </Fieldset>

        <div class="rounded-lg border border-divider">
          <p v-if="membersStore.error" class="banner-danger">{{ membersStore.error }}</p>
          <DataTable
            class="app-data-table"
            v-model:selection="selectedMembers"
            data-key="username"
            :value="filteredMembers"
            :loading="membersStore.loading"
            paginator
            :rows="paginationCompact.rows"
            :rows-per-page-options="paginationCompact.rowsPerPageOptions"
            :paginator-template="paginationCompact.paginatorTemplate"
            :current-page-report-template="paginationCompact.currentPageReportTemplate"
            empty-message="No members match the search conditions."
            :table-style="{ minWidth: '520px' }"
          >
            <Column selection-mode="multiple" header-class="w-12" body-class="text-center" />
            <Column field="username" header="Username" body-class="font-bold text-ink" />
            <Column field="name" header="Name" />
          </DataTable>
        </div>
      </div>

      <!-- Stage 2: confirm selected members -->
      <div v-else class="grid gap-3">
        <p class="text-sm text-muted">{{ pendingMembers.length }} member(s) selected.</p>
        <div class="max-h-[360px] overflow-auto rounded-lg border border-divider">
          <DataTable class="app-data-table" :value="pendingMembers" data-key="username" :table-style="{ minWidth: '520px' }">
            <Column field="username" header="Username" body-class="font-bold text-ink" />
            <Column field="name" header="Name" />
          </DataTable>
        </div>
      </div>

      <template #footer>
        <DialogFooter
          v-if="!isConfirmStage"
          confirm-icon="pi pi-check"
          :confirm-label="`Select (${selectedMembers.length})`"
          :confirm-disabled="selectedMembers.length === 0"
          @cancel="isSearchHelpOpen = false"
          @confirm="goToConfirm"
        />
        <DialogFooter
          v-else
          cancel-label="Back"
          cancel-icon="pi pi-arrow-left"
          confirm-icon="pi pi-check"
          confirm-label="Confirm"
          @cancel="isConfirmStage = false"
          @confirm="confirmMembers"
        />
      </template>
    </Dialog>
  </section>
</template>
