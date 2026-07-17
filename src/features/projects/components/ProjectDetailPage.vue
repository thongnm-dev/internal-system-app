<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import DataTable from "primevue/datatable";
import Column from "primevue/column";
import Dialog from "primevue/dialog";
import Fieldset from "primevue/fieldset";
import Button from "primevue/button";
import InputText from "primevue/inputtext";
import { friendlyError } from "@/tauri/commands/_base";
import { createProject, updateProject, getBacklogProjectByKey, getProjectDetail } from "@/tauri/commands/project";
import { useToast } from "@/shared/composables/useToast";
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

const emptyForm: ProjectForm = { id: null, code: "", name: "", client: "", backlogKey: "", backlogCode: "", backlogName: "" };
const memberSearchHelpItems: ProjectMember[] = [
  { username: "thongnm", name: "Thong Nguyen" },
  { username: "annatn", name: "Anna Tran" },
  { username: "binhpt", name: "Binh Pham" },
  { username: "hanhld", name: "Hanh Le" },
  { username: "minhvo", name: "Minh Vo" },
];

const route = useRoute();
const router = useRouter();
const toast = useToast();
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

function normalize(v: string) { return v.trim().toLocaleLowerCase(); }

const filteredMembers = computed(() => {
  const kw = normalize(memberKeyword.value);
  const selected = new Set(members.value.map((m) => m.username));
  return memberSearchHelpItems.filter((m) => !selected.has(m.username) && (!kw || normalize(`${m.username} ${m.name}`).includes(kw)));
});

function addMember(member: ProjectMember) {
  members.value = [...members.value, member].sort((a, b) => a.username.localeCompare(b.username));
  memberKeyword.value = "";
  isSearchHelpOpen.value = false;
}

function removeMember(username: string) {
  members.value = members.value.filter((m) => m.username !== username);
}

function updateForm(key: keyof ProjectForm, value: string) {
  form.value = { ...form.value, [key]: value };
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
  isSaving.value = true;
  saveError.value = "";
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
    <p v-if="loadError" class="mt-4 rounded-lg border border-amber-200 bg-amber-50 p-3 text-sm text-amber-800">{{ loadError }}</p>
    <p v-if="saveError" class="mt-4 rounded-lg border border-red-200 bg-red-50 p-3 text-sm text-red-800">{{ saveError }}</p>

    <div class="mt-4 grid gap-4">
      <Fieldset class="rounded-lg border border-divider bg-panel p-4 shadow-md fieldset-nested" toggleable legend="Project Information">
        <div class="grid gap-3 md:grid-cols-2">
          <label>
            <span class="text-xs font-bold text-muted">Project ID</span>
            <InputText class="mt-1 w-full" placeholder="Auto generated" readonly :model-value="String(form.id ?? '')" />
          </label>
          <label>
            <span class="text-xs font-bold text-muted">Project Code</span>
            <InputText class="mt-1 w-full" placeholder="Project code" :model-value="form.code" @update:model-value="updateForm('code', $event as string)" />
          </label>
          <label class="md:col-span-2">
            <span class="text-xs font-bold text-muted">Project Name</span>
            <InputText class="mt-1 w-full" placeholder="Project name" :model-value="form.name" @update:model-value="updateForm('name', $event as string)" />
          </label>
          <label class="md:col-span-2">
            <span class="text-xs font-bold text-muted">Client</span>
            <InputText class="mt-1 w-full" placeholder="Client name" :model-value="form.client" @update:model-value="updateForm('client', $event as string)" />
          </label>
          <Fieldset class="rounded-lg border border-divider p-4 md:col-span-2 fieldset-nested" legend="Backlog" toggleable>
            <div class="grid gap-3 md:grid-cols-2">
              <div>
                <span class="text-xs font-bold text-muted">Backlog Key</span>
                <div class="group/key mt-1 flex rounded-md ring-emerald-100 focus-within:ring-2">
                  <InputText
                    class="min-w-0 flex-1 !rounded-r-none"
                    placeholder="BACKLOG_KEY"
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
              </div>
              <label>
                <span class="text-xs font-bold text-muted">Backlog Code</span>
                <InputText class="mt-1 w-full" disabled :placeholder="isBacklogLookupLoading ? 'Loading...' : ''" :model-value="form.backlogCode" />
              </label>
              <label class="md:col-span-2">
                <span class="text-xs font-bold text-muted">Backlog Name</span>
                <InputText class="mt-1 w-full" disabled :placeholder="isBacklogLookupLoading ? 'Loading...' : ''" :model-value="form.backlogName" />
              </label>
              <p v-if="backlogLookupError" class="text-sm text-red-600 md:col-span-2">{{ backlogLookupError }}</p>
            </div>
          </Fieldset>
        </div>
      </Fieldset>

      <Fieldset class="rounded-lg border border-divider bg-panel p-4 shadow-md fieldset-nested" legend="Members" toggleable>
        <div class="flex flex-wrap items-center justify-between gap-3">
          <h3 class="font-bold text-ink">Members</h3>
          <Button icon="pi pi-plus" label="Add member" @click="isSearchHelpOpen = true" />
        </div>
        <div class="mt-4 overflow-auto rounded-lg border border-divider">
          <DataTable class="app-data-table" empty-message="No members selected." :table-style="{ minWidth: '560px' }" :value="members">
            <Column field="username" header="Username" body-class="font-bold text-ink" />
            <Column field="name" header="Name" />
            <Column header="Action" body-class="text-center" header-class="w-20 text-center">
              <template #body="{ data }">
                <Button icon="pi pi-trash" severity="danger" text rounded size="small" title="Remove member" @click="removeMember(data.username)" />
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
          <h3 class="font-bold text-ink">Search help members</h3>
          <p class="mt-1 text-sm text-muted">Search and select members to add to the project.</p>
        </div>
      </template>

      <div>
        <label>
          <span class="text-xs font-bold text-muted">Keyword</span>
          <InputText class="mt-1 w-full" placeholder="Search username or name" v-model="memberKeyword" />
        </label>
        <div class="mt-4 max-h-[360px] overflow-auto rounded-lg border border-divider">
          <DataTable class="app-data-table" empty-message="No members match the search conditions." :table-style="{ minWidth: '520px' }" :value="filteredMembers">
            <Column field="username" header="Username" body-class="font-bold text-ink" />
            <Column field="name" header="Name" />
            <Column header="Select" body-class="text-center" header-class="w-20 text-center">
              <template #body="{ data }">
                <Button icon="pi pi-plus" rounded size="small" title="Select member" @click="addMember(data)" />
              </template>
            </Column>
          </DataTable>
        </div>
      </div>
    </Dialog>
  </section>
</template>
