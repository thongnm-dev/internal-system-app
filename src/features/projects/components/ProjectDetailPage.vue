<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useRoute } from "vue-router";
import DataTable from "primevue/datatable";
import Column from "primevue/column";
import Dialog from "primevue/dialog";
import Fieldset from "primevue/fieldset";
import { friendlyError, getBacklogProjectByKey, getProjectDetail, type ProjectMember } from "@/tauri/commands";
type ProjectForm = { backlogProjectID: string; backlogProjectKey: string; backlogProjectName: string; projectID: string; projectCode: string; projectName: string };

const emptyForm: ProjectForm = { backlogProjectID: "", backlogProjectKey: "", backlogProjectName: "", projectID: "", projectCode: "", projectName: "" };
const memberSearchHelpItems: ProjectMember[] = [
  { username: "thongnm", name: "Thong Nguyen" },
  { username: "annatn", name: "Anna Tran" },
  { username: "binhpt", name: "Binh Pham" },
  { username: "hanhld", name: "Hanh Le" },
  { username: "minhvo", name: "Minh Vo" },
];

const route = useRoute();
const projectID = (route.params.id as string) || null;

const form = ref<ProjectForm>({ ...emptyForm });
const isLoading = ref(false);
const isBacklogLookupLoading = ref(false);
const isSearchHelpOpen = ref(false);
const backlogLookupError = ref("");
const loadError = ref("");
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
    form.value = { backlogProjectID: project.backlog_project_id ? String(project.backlog_project_id) : "", backlogProjectKey: project.backlog_project_key ?? "", backlogProjectName: project.backlog_project_name ?? "", projectID: String(project.project_id), projectCode: project.project_code, projectName: project.project_name };
    members.value = project.members;
  } catch (e) {
    form.value = { ...emptyForm, projectID: projectID ?? "" };
    loadError.value = friendlyError(e);
  } finally {
    isLoading.value = false;
  }
});

async function fetchBacklogProject(key: string) {
  const trimmed = key.trim();
  if (!trimmed) {
    backlogLookupError.value = "";
    form.value = { ...form.value, backlogProjectID: "", backlogProjectName: "" };
    return;
  }
  isBacklogLookupLoading.value = true;
  backlogLookupError.value = "";
  try {
    const project = await getBacklogProjectByKey(trimmed);
    form.value = { ...form.value, backlogProjectID: String(project.project_id), backlogProjectKey: project.project_key, backlogProjectName: project.project_name };
  } catch (e) {
    form.value = { ...form.value, backlogProjectID: "", backlogProjectName: "" };
    backlogLookupError.value = friendlyError(e);
  } finally {
    isBacklogLookupLoading.value = false;
  }
}

function reloadBacklogProject() {
  fetchBacklogProject(form.value.backlogProjectKey);
}

let backlogLookupTimeout: number | undefined;
watch(() => form.value.backlogProjectKey, (key) => {
  clearTimeout(backlogLookupTimeout);
  backlogLookupTimeout = window.setTimeout(() => fetchBacklogProject(key), 500);
});
</script>

<template>
  <section class="min-h-0 flex-1 overflow-auto rounded-lg border border-divider bg-panel p-4 shadow-sm">
    <div class="flex items-center justify-end gap-4">
      <button class="flex h-9 items-center gap-2 rounded-md bg-brand px-3 text-sm font-bold text-white hover:opacity-90" type="button"><i class="pi pi-save" />Save</button>
    </div>

    <p v-if="isLoading" class="mt-4 text-sm text-muted">Loading project information...</p>
    <p v-if="loadError" class="mt-4 rounded-lg border border-amber-200 bg-amber-50 p-3 text-sm text-amber-800">{{ loadError }}</p>

    <div class="mt-4 grid gap-4">
      <Fieldset class="rounded-lg border border-divider bg-panel p-4 shadow-md fieldset-nested" toggleable legend="Project Information">
        <div class="grid gap-3 md:grid-cols-2">
          <label>
            <span class="text-xs font-bold text-muted">Project ID</span>
            <input class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none" placeholder="Auto generated" readonly :value="form.projectID" />
          </label>
          <label>
            <span class="text-xs font-bold text-muted">Project Code</span>
            <input class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100" placeholder="Project code" :value="form.projectCode" @input="updateForm('projectCode', ($event.target as HTMLInputElement).value)" />
          </label>
          <label class="md:col-span-2">
            <span class="text-xs font-bold text-muted">Project Name</span>
            <input class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100" placeholder="Project name" :value="form.projectName" @input="updateForm('projectName', ($event.target as HTMLInputElement).value)" />
          </label>
          <Fieldset class="rounded-lg border border-divider p-4 md:col-span-2 fieldset-nested" legend="Backlog" toggleable>
            <div class="grid gap-3 md:grid-cols-2">
              <div>
                <span class="text-xs font-bold text-muted">Backlog Project Key</span>
                <div class="group/key mt-1 flex rounded-md ring-emerald-100 focus-within:ring-2">
                  <input
                    class="h-10 min-w-0 flex-1 rounded-l-md border border-r-0 border-divider bg-panel px-3 text-sm text-ink outline-none group-hover/key:border-brand group-focus-within/key:border-brand"
                    placeholder="BACKLOG_PROJECT_KEY"
                    :value="form.backlogProjectKey"
                    @input="updateForm('backlogProjectKey', ($event.target as HTMLInputElement).value)"
                  />
                  <button
                    class="flex h-10 w-10 shrink-0 items-center justify-center rounded-r-md border border-divider bg-canvas text-secondary group-hover/key:border-brand group-focus-within/key:border-brand hover:text-brand disabled:cursor-not-allowed disabled:opacity-50"
                    type="button"
                    title="Reload from Backlog API"
                    :disabled="isBacklogLookupLoading || !form.backlogProjectKey.trim()"
                    @click="reloadBacklogProject"
                  >
                    <i :class="['pi pi-refresh', isBacklogLookupLoading ? 'animate-spin' : '']" />
                  </button>
                </div>
              </div>
              <label>
                <span class="text-xs font-bold text-muted">Backlog Project ID</span>
                <input class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none" :placeholder="isBacklogLookupLoading ? 'Loading...' : 'Auto fetched from Backlog'" readonly :value="form.backlogProjectID" />
              </label>
              <label class="md:col-span-2">
                <span class="text-xs font-bold text-muted">Backlog Project Name</span>
                <input class="mt-1 h-10 w-full rounded-md border border-divider bg-panel px-3 text-sm text-ink outline-none" :placeholder="isBacklogLookupLoading ? 'Loading...' : 'Auto fetched from Backlog'" readonly :value="form.backlogProjectName" />
              </label>
              <p v-if="backlogLookupError" class="text-sm text-red-600 md:col-span-2">{{ backlogLookupError }}</p>
            </div>
          </Fieldset>
        </div>
      </Fieldset>

      <Fieldset class="rounded-lg border border-divider bg-panel p-4 shadow-md fieldset-nested" legend="Members" toggleable>
        <div class="flex flex-wrap items-center justify-between gap-3">
          <h3 class="font-bold text-ink">Members</h3>
          <button class="flex h-10 items-center gap-2 rounded-md bg-brand px-4 text-sm font-bold text-white hover:opacity-90" type="button" @click="isSearchHelpOpen = true"><i class="pi pi-plus" />Add member</button>
        </div>
        <div class="mt-4 overflow-auto rounded-lg border border-divider">
          <DataTable class="app-data-table" empty-message="No members selected." :table-style="{ minWidth: '560px' }" :value="members">
            <Column field="username" header="Username" body-class="font-bold text-ink" />
            <Column field="name" header="Name" />
            <Column header="Action" body-class="text-center" header-class="w-20 text-center">
              <template #body="{ data }">
                <button class="inline-flex h-8 w-8 items-center justify-center rounded-md border border-divider bg-panel text-secondary hover:bg-canvas" type="button" title="Remove member" @click="removeMember(data.username)"><i class="pi pi-trash" /></button>
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
          <div class="mt-1 flex h-10 items-center rounded-md border border-divider bg-panel px-3 focus-within:border-brand focus-within:ring-2 focus-within:ring-emerald-100">
            <i class="pi pi-search shrink-0 text-muted" />
            <input class="h-full min-w-0 flex-1 border-0 px-2 text-sm text-ink outline-none" placeholder="Search username or name" type="search" v-model="memberKeyword" />
          </div>
        </label>
        <div class="mt-4 max-h-[360px] overflow-auto rounded-lg border border-divider">
          <DataTable class="app-data-table" empty-message="No members match the search conditions." :table-style="{ minWidth: '520px' }" :value="filteredMembers">
            <Column field="username" header="Username" body-class="font-bold text-ink" />
            <Column field="name" header="Name" />
            <Column header="Select" body-class="text-center" header-class="w-20 text-center">
              <template #body="{ data }">
                <button class="inline-flex h-8 w-8 items-center justify-center rounded-md bg-brand text-white hover:opacity-90" type="button" title="Select member" @click="addMember(data)"><i class="pi pi-plus" /></button>
              </template>
            </Column>
          </DataTable>
        </div>
      </div>
    </Dialog>
  </section>
</template>
