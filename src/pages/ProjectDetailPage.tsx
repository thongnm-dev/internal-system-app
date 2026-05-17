import { ArrowLeft, Plus, Save, Search, Trash2, X } from "lucide-react";
import { Button } from "primereact/button";
import { Column } from "primereact/column";
import { DataTable } from "primereact/datatable";
import { Fieldset } from "primereact/fieldset";
import { InputText } from "primereact/inputtext";
import { useEffect, useMemo, useState } from "react";
import { friendlyError, safeInvoke } from "../core/tauriRuntime";

type ProjectDetailPageProps = {
  projectID: string | null;
  onBack: () => void;
};

type ProjectMember = {
  username: string;
  name: string;
};

type ProjectDetail = {
  backlog_project_id?: string | number;
  backlog_project_key?: string;
  backlog_project_name?: string;
  project_id: string | number;
  project_code: string;
  project_name: string;
  members: ProjectMember[];
};

type BacklogProjectLookup = {
  project_id: string | number;
  project_key: string;
  project_name: string;
};

type ProjectForm = {
  backlogProjectID: string;
  backlogProjectKey: string;
  backlogProjectName: string;
  projectID: string;
  projectCode: string;
  projectName: string;
};

const emptyProjectForm: ProjectForm = {
  backlogProjectID: "",
  backlogProjectKey: "",
  backlogProjectName: "",
  projectID: "",
  projectCode: "",
  projectName: "",
};

const memberSearchHelpItems: ProjectMember[] = [
  { username: "thongnm", name: "Thong Nguyen" },
  { username: "annatn", name: "Anna Tran" },
  { username: "binhpt", name: "Binh Pham" },
  { username: "hanhld", name: "Hanh Le" },
  { username: "minhvo", name: "Minh Vo" },
];

export function ProjectDetailPage({ projectID, onBack }: ProjectDetailPageProps) {
  const [form, setForm] = useState<ProjectForm>(emptyProjectForm);
  const [isLoading, setIsLoading] = useState(false);
  const [isBacklogLookupLoading, setIsBacklogLookupLoading] = useState(false);
  const [isSearchHelpOpen, setIsSearchHelpOpen] = useState(false);
  const [backlogLookupError, setBacklogLookupError] = useState("");
  const [loadError, setLoadError] = useState("");
  const [memberKeyword, setMemberKeyword] = useState("");
  const [members, setMembers] = useState<ProjectMember[]>([]);
  const isCreateMode = !projectID;

  useEffect(() => {
    if (!projectID) {
      setForm(emptyProjectForm);
      setMembers([]);
      setLoadError("");
      return;
    }

    let isActive = true;
    setIsLoading(true);
    setLoadError("");

    safeInvoke<ProjectDetail>("get_project_detail", { projectId: projectID })
      .then((project) => {
        if (!isActive) {
          return;
        }

        setForm({
          backlogProjectID: project.backlog_project_id ? String(project.backlog_project_id) : "",
          backlogProjectKey: project.backlog_project_key ?? "",
          backlogProjectName: project.backlog_project_name ?? "",
          projectID: String(project.project_id),
          projectCode: project.project_code,
          projectName: project.project_name,
        });
        setMembers(project.members);
      })
      .catch((error) => {
        if (!isActive) {
          return;
        }

        setForm({ ...emptyProjectForm, projectID });
        setMembers([]);
        setLoadError(friendlyError(error));
      })
      .finally(() => {
        if (isActive) {
          setIsLoading(false);
        }
      });

    return () => {
      isActive = false;
    };
  }, [projectID]);

  useEffect(() => {
    const projectKey = form.backlogProjectKey.trim();

    if (!projectKey) {
      setBacklogLookupError("");
      setIsBacklogLookupLoading(false);
      setForm((current) => ({ ...current, backlogProjectID: "", backlogProjectName: "" }));
      return;
    }

    let isActive = true;
    const timeoutID = window.setTimeout(() => {
      setIsBacklogLookupLoading(true);
      setBacklogLookupError("");

      safeInvoke<BacklogProjectLookup>("get_backlog_project_by_key", { projectKey })
        .then((project) => {
          if (!isActive) {
            return;
          }

          setForm((current) => ({
            ...current,
            backlogProjectID: String(project.project_id),
            backlogProjectKey: project.project_key,
            backlogProjectName: project.project_name,
          }));
        })
        .catch((error) => {
          if (!isActive) {
            return;
          }

          setForm((current) => ({ ...current, backlogProjectID: "", backlogProjectName: "" }));
          setBacklogLookupError(friendlyError(error));
        })
        .finally(() => {
          if (isActive) {
            setIsBacklogLookupLoading(false);
          }
        });
    }, 500);

    return () => {
      isActive = false;
      window.clearTimeout(timeoutID);
    };
  }, [form.backlogProjectKey]);

  const filteredMembers = useMemo(() => {
    const keyword = normalize(memberKeyword);
    const selectedUsernames = new Set(members.map((member) => member.username));

    return memberSearchHelpItems.filter((member) => {
      if (selectedUsernames.has(member.username)) {
        return false;
      }

      if (!keyword) {
        return true;
      }

      return normalize(`${member.username} ${member.name}`).includes(keyword);
    });
  }, [memberKeyword, members]);

  const addMember = (member: ProjectMember) => {
    setMembers((current) => [...current, member].sort((a, b) => a.username.localeCompare(b.username)));
    setMemberKeyword("");
    setIsSearchHelpOpen(false);
  };

  const removeMember = (username: string) => {
    setMembers((current) => current.filter((member) => member.username !== username));
  };

  const updateForm = (key: keyof ProjectForm, value: string) => {
    setForm((current) => ({ ...current, [key]: value }));
  };

  return (
    <section className="min-h-0 flex-1 overflow-auto rounded-lg border border-stone-200 bg-panel p-4 shadow-sm">
      <div className="flex items-center justify-between gap-4">
        <Button
          className="flex h-9 items-center gap-2 rounded-md border border-slate-300 bg-white px-3 text-sm font-bold text-slate-700 hover:bg-slate-50"
          type="button"
          onClick={onBack}
        >
          <ArrowLeft className="h-4 w-4" />
          Back
        </Button>
        <Button
          className="flex h-9 items-center gap-2 rounded-md bg-brand px-3 text-sm font-bold text-white hover:opacity-90"
          type="button"
        >
          <Save className="h-4 w-4" />
          Save
        </Button>
      </div>

      {isLoading ? <p className="mt-4 text-sm text-slate-500">Loading project information...</p> : null}
      {loadError ? (
        <p className="mt-4 rounded-lg border border-amber-200 bg-amber-50 p-3 text-sm text-amber-800">{loadError}</p>
      ) : null}

      <div className="mt-4 grid gap-4">
        <Fieldset className="rounded-lg border border-slate-200 bg-white p-4 shadow-md fieldset-nested" toggleable legend="Thông tin chung">
          <div className="grid gap-3 md:grid-cols-2">
            <label>
              <span className="text-xs font-bold text-slate-500">Project ID</span>
              <InputText
                className="mt-1 h-10 w-full rounded-md border border-slate-300 bg-white px-3 text-sm text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                placeholder="Auto generated"
                readOnly
                value={form.projectID}
              />
            </label>
            <label>
              <span className="text-xs font-bold text-slate-500">Project Code</span>
              <InputText
                className="mt-1 h-10 w-full rounded-md border border-slate-300 bg-white px-3 text-sm text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                placeholder="Project code"
                value={form.projectCode}
                onChange={(event) => updateForm("projectCode", event.target.value)}
              />
            </label>
            <label className="md:col-span-2">
              <span className="text-xs font-bold text-slate-500">Project Name</span>
              <InputText
                className="mt-1 h-10 w-full rounded-md border border-slate-300 bg-white px-3 text-sm text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                placeholder="Project name"
                value={form.projectName}
                onChange={(event) => updateForm("projectName", event.target.value)}
              />
            </label>
            <Fieldset className="rounded-lg border border-stone-200 p-4 md:col-span-2 fieldset-nested" legend="Backlog" toggleable>
              <div className="grid gap-3 md:grid-cols-2">
                <label>
                  <span className="text-xs font-bold text-slate-500">Backlog Project Key</span>
                  <InputText
                    className="mt-1 h-10 w-full rounded-md border border-slate-300 bg-white px-3 text-sm text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                    placeholder="BACKLOG_PROJECT_KEY"
                    value={form.backlogProjectKey}
                    onChange={(event) => updateForm("backlogProjectKey", event.target.value)}
                  />
                </label>
                <label>
                  <span className="text-xs font-bold text-slate-500">Backlog Project ID</span>
                  <InputText
                    className="mt-1 h-10 w-full rounded-md border border-slate-300 bg-white px-3 text-sm text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                    placeholder={isBacklogLookupLoading ? "Loading..." : "Auto fetched from Backlog"}
                    readOnly
                    value={form.backlogProjectID}
                  />
                </label>
                <label className="md:col-span-2">
                  <span className="text-xs font-bold text-slate-500">Backlog Project Name</span>
                  <InputText
                    className="mt-1 h-10 w-full rounded-md border border-slate-300 bg-white px-3 text-sm text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                    placeholder={isBacklogLookupLoading ? "Loading..." : "Auto fetched from Backlog"}
                    readOnly
                    value={form.backlogProjectName}
                  />
                </label>
                {backlogLookupError ? <p className="text-sm text-red-600 md:col-span-2">{backlogLookupError}</p> : null}
              </div>
            </Fieldset>
          </div>
        </Fieldset>

        <Fieldset className="rounded-lg border border-stone-200 bg-white p-4 shadow-md fieldset-nested" legend="Thành viên" toggleable>
          <div className="flex flex-wrap items-center justify-between gap-3">
            <h3 className="font-bold text-slate-800">Thanh vien</h3>
            <Button
              className="flex h-10 items-center gap-2 rounded-md bg-brand px-4 text-sm font-bold text-white hover:opacity-90"
              type="button"
              onClick={() => setIsSearchHelpOpen(true)}
            >
              <Plus className="h-4 w-4" />
              Add member
            </Button>
          </div>

          <div className="mt-4 overflow-auto rounded-lg border border-stone-200">
            <DataTable
              className="app-data-table"
              emptyMessage="Chua chon thanh vien."
              tableStyle={{ minWidth: "560px" }}
              value={members}
            >
              <Column field="username" header="Username" bodyClassName="font-bold text-ink" />
              <Column field="name" header="Ten" />
              <Column header="Action" body={(member: ProjectMember) => removeMemberBody(member, removeMember)} bodyClassName="text-center" headerClassName="w-20 text-center" />
            </DataTable>
          </div>
        </Fieldset>
      </div>

      {isSearchHelpOpen ? (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/40 p-4">
          <section className="w-full max-w-2xl rounded-lg bg-white shadow-xl">
            <div className="flex items-center justify-between gap-4 border-b border-stone-200 px-4 py-3">
              <h3 className="font-bold">Search help thanh vien</h3>
              <Button
                className="flex h-9 w-9 items-center justify-center rounded-md border border-slate-300 bg-white text-slate-600 hover:bg-slate-50"
                type="button"
                title="Close"
                onClick={() => setIsSearchHelpOpen(false)}
              >
                <X className="h-4 w-4" />
              </Button>
            </div>

            <div className="p-4">
              <label>
                <span className="text-xs font-bold text-slate-500">Keyword</span>
                <div className="mt-1 flex h-10 items-center rounded-md border border-slate-300 bg-white px-3 focus-within:border-brand focus-within:ring-2 focus-within:ring-emerald-100">
                  <Search className="h-4 w-4 shrink-0 text-slate-400" />
                  <InputText
                    className="h-full min-w-0 flex-1 border-0 px-2 text-sm text-slate-900 outline-none"
                    placeholder="Search username or name"
                    type="search"
                    value={memberKeyword}
                    onChange={(event) => setMemberKeyword(event.target.value)}
                  />
                </div>
              </label>

              <div className="mt-4 max-h-[360px] overflow-auto rounded-lg border border-stone-200">
                <DataTable
                  className="app-data-table"
                  emptyMessage="No members match the search conditions."
                  tableStyle={{ minWidth: "520px" }}
                  value={filteredMembers}
                >
                  <Column field="username" header="Username" bodyClassName="font-bold text-ink" />
                  <Column field="name" header="Ten" />
                  <Column header="Select" body={(member: ProjectMember) => addMemberBody(member, addMember)} bodyClassName="text-center" headerClassName="w-20 text-center" />
                </DataTable>
              </div>
            </div>
          </section>
        </div>
      ) : null}
    </section>
  );
}

function removeMemberBody(member: ProjectMember, removeMember: (username: string) => void) {
  return (
    <Button
      className="inline-flex h-8 w-8 items-center justify-center rounded-md border border-slate-300 bg-white text-slate-600 hover:bg-slate-50"
      type="button"
      title="Remove member"
      onClick={() => removeMember(member.username)}
    >
      <Trash2 className="h-4 w-4" />
    </Button>
  );
}

function addMemberBody(member: ProjectMember, addMember: (member: ProjectMember) => void) {
  return (
    <Button
      className="inline-flex h-8 w-8 items-center justify-center rounded-md bg-brand text-white hover:opacity-90"
      type="button"
      title="Select member"
      onClick={() => addMember(member)}
    >
      <Plus className="h-4 w-4" />
    </Button>
  );
}

function normalize(value: string) {
  return value.trim().toLocaleLowerCase();
}
