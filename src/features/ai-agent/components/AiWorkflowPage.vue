<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import Button from "primevue/button";
import Dialog from "primevue/dialog";
import InputText from "primevue/inputtext";
import Select from "primevue/select";
import IconPickerDialog from "@/shared/components/IconPickerDialog.vue";
import { useAiWorkflow } from "../composables/useAiWorkflow";
import type { WorkflowStepType } from "@/_/types/ai-workflow";
import { STEP_TYPE_META } from "@/_/types/ai-workflow";

const ctrl = useAiWorkflow();

// --- Sidebar resize (pattern from AiChatPage.vue) ---
const SIDEBAR_MIN = 200;
const SIDEBAR_MAX = 400;
const sidebarWidth = ref(280);
const sidebarCollapsed = ref(false);
let cleanupDrag: (() => void) | null = null;

function toggleSidebar() {
  sidebarCollapsed.value = !sidebarCollapsed.value;
}

function startDrag(event: MouseEvent) {
  event.preventDefault();
  const startX = event.clientX;
  const startWidth = sidebarWidth.value;
  function onMove(ev: MouseEvent) {
    sidebarWidth.value = Math.min(SIDEBAR_MAX, Math.max(SIDEBAR_MIN, startWidth + (ev.clientX - startX)));
  }
  function onUp() {
    document.removeEventListener("mousemove", onMove);
    document.removeEventListener("mouseup", onUp);
    document.body.style.userSelect = "";
    cleanupDrag = null;
  }
  document.body.style.userSelect = "none";
  document.addEventListener("mousemove", onMove);
  document.addEventListener("mouseup", onUp);
  cleanupDrag = onUp;
}

onBeforeUnmount(() => cleanupDrag?.());

// --- Workflow dialog ---
const showWorkflowDialog = ref(false);
const editingWorkflowId = ref<number | null>(null);
const wfName = ref("");
const wfDescription = ref("");

function openCreateWorkflowDialog() {
  editingWorkflowId.value = null;
  wfName.value = "";
  wfDescription.value = "";
  showWorkflowDialog.value = true;
}

function openEditWorkflowDialog() {
  const wf = ctrl.activeWorkflow.value;
  if (!wf) return;
  editingWorkflowId.value = wf.id;
  wfName.value = wf.name;
  wfDescription.value = wf.description;
  showWorkflowDialog.value = true;
}

async function saveWorkflow() {
  const name = wfName.value.trim();
  if (!name) return;
  if (editingWorkflowId.value) {
    await ctrl.updateWorkflow(editingWorkflowId.value, { name, description: wfDescription.value.trim() });
  } else {
    await ctrl.createWorkflow(name, wfDescription.value.trim());
  }
  showWorkflowDialog.value = false;
}

// --- Step dialog ---
const showStepDialog = ref(false);
const editingStepId = ref<number | null>(null);
const afterStepId = ref<number | null>(null);
const stepName = ref("");
const stepType = ref<WorkflowStepType>("custom");
const stepDescription = ref("");
const stepIcon = ref("pi pi-cog");

const stepTypeOptions = Object.entries(STEP_TYPE_META).map(([value, meta]) => ({
  label: meta.label,
  value: value as WorkflowStepType,
  icon: meta.icon,
}));

function openAddStepDialog(after: number | null) {
  editingStepId.value = null;
  afterStepId.value = after;
  stepName.value = "";
  stepType.value = "custom";
  stepDescription.value = "";
  stepIcon.value = STEP_TYPE_META.custom.icon;
  showStepDialog.value = true;
}

function openEditStepDialog(stepId: number) {
  const step = ctrl.activeSteps.value.find((s) => s.id === stepId);
  if (!step) return;
  editingStepId.value = step.id;
  afterStepId.value = null;
  stepName.value = step.name;
  stepType.value = step.type;
  stepDescription.value = step.description;
  stepIcon.value = step.icon;
  showStepDialog.value = true;
}

function onStepTypeChange() {
  const meta = STEP_TYPE_META[stepType.value];
  if (meta) stepIcon.value = meta.icon;
}

async function saveStep() {
  const name = stepName.value.trim();
  if (!name) return;
  const data = { name, type: stepType.value, description: stepDescription.value.trim(), icon: stepIcon.value };
  if (editingStepId.value) {
    await ctrl.updateStep(editingStepId.value, data);
  } else {
    await ctrl.addStep(afterStepId.value, data);
  }
  showStepDialog.value = false;
}

// --- Delete confirmation ---
const showDeleteDialog = ref(false);
const deleteTarget = ref<{ type: "workflow" | "step"; id: number; name: string } | null>(null);

function confirmDeleteWorkflow() {
  const wf = ctrl.activeWorkflow.value;
  if (!wf) return;
  deleteTarget.value = { type: "workflow", id: wf.id, name: wf.name };
  showDeleteDialog.value = true;
}

function confirmDeleteStep(stepId: number) {
  const step = ctrl.activeSteps.value.find((s) => s.id === stepId);
  if (!step) return;
  deleteTarget.value = { type: "step", id: step.id, name: step.name };
  showDeleteDialog.value = true;
}

async function executeDelete() {
  if (!deleteTarget.value) return;
  if (deleteTarget.value.type === "workflow") {
    await ctrl.deleteWorkflow(deleteTarget.value.id);
  } else {
    await ctrl.deleteStep(deleteTarget.value.id);
  }
  showDeleteDialog.value = false;
  deleteTarget.value = null;
}

// --- SVG Arrow computation ---
const nodesContainer = ref<HTMLElement | null>(null);
const arrowPaths = ref<{ id: string; d: string }[]>([]);
let resizeObserver: ResizeObserver | null = null;

function updateArrows() {
  const container = nodesContainer.value;
  const steps = ctrl.activeSteps.value;
  if (!container || steps.length < 2) {
    arrowPaths.value = [];
    return;
  }

  const containerRect = container.getBoundingClientRect();
  const paths: { id: string; d: string }[] = [];

  for (let i = 0; i < steps.length - 1; i++) {
    const fromEl = container.querySelector(`[data-step-id="${steps[i].id}"]`) as HTMLElement | null;
    const toEl = container.querySelector(`[data-step-id="${steps[i + 1].id}"]`) as HTMLElement | null;
    if (!fromEl || !toEl) continue;

    const fromRect = fromEl.getBoundingClientRect();
    const toRect = toEl.getBoundingClientRect();

    const x1 = fromRect.right - containerRect.left;
    const y1 = fromRect.top + fromRect.height / 2 - containerRect.top;
    const x2 = toRect.left - containerRect.left;
    const y2 = toRect.top + toRect.height / 2 - containerRect.top;

    const sameRow = Math.abs(y1 - y2) < 30;
    let d: string;
    if (sameRow) {
      const cx = (x1 + x2) / 2;
      d = `M ${x1} ${y1} C ${cx} ${y1}, ${cx} ${y2}, ${x2} ${y2}`;
    } else {
      d = `M ${x1} ${y1} C ${x1 + 50} ${y1}, ${x2 - 50} ${y2}, ${x2} ${y2}`;
    }
    paths.push({ id: `arrow-${steps[i].id}-${steps[i + 1].id}`, d });
  }

  arrowPaths.value = paths;
}

watch(
  () => ctrl.activeId.value,
  async (id) => {
    if (id !== null) {
      await nextTick();
      loadPositions(id);
    }
    await nextTick();
    updateArrows();
  },
);

watch(
  () => ctrl.activeSteps.value.length,
  async () => {
    ctrl.activeSteps.value.forEach((s) => ensureStepPosition(s.id));
    await nextTick();
    updateArrows();
  },
);

onMounted(() => {
  if (ctrl.activeId.value !== null) {
    loadPositions(ctrl.activeId.value);
  }
  void nextTick(() => updateArrows());
  if (nodesContainer.value) {
    resizeObserver = new ResizeObserver(() => updateArrows());
    resizeObserver.observe(nodesContainer.value);
  }
});

onBeforeUnmount(() => resizeObserver?.disconnect());

// --- Helpers ---
function formatDate(iso: string): string {
  return new Date(iso).toLocaleDateString([], { month: "short", day: "numeric" });
}

function stepTypeBadgeClass(type: WorkflowStepType): string {
  return STEP_TYPE_META[type]?.badgeClass ?? "bg-canvas text-muted";
}

function stepTypeLabel(type: WorkflowStepType): string {
  return STEP_TYPE_META[type]?.label ?? "Custom";
}

const showIconPicker = ref(false);

// --- Free-form canvas node positioning ---
const NODE_W = 208;
const NODE_H = 140;
const H_GAP = 120;
const V_GAP = 100;
const NODES_PER_ROW = 3;

type NodePos = { x: number; y: number };
const nodePositions = ref<Record<number, NodePos>>({});
const draggingNodeId = ref<number | null>(null);
let nodeDragStartX = 0;
let nodeDragStartY = 0;
let nodeDragOrigX = 0;
let nodeDragOrigY = 0;

function loadPositions(workflowId: number) {
  const wf = ctrl.workflows.value.find((w) => w.id === workflowId);
  if (wf && Object.keys(wf.layout).length > 0) {
    nodePositions.value = wf.layout as Record<number, NodePos>;
  } else {
    autoLayout();
  }
}

function savePositions() {
  const wf = ctrl.activeWorkflow.value;
  if (!wf) return;
  ctrl.saveLayout(wf.id, nodePositions.value as Record<string, { x: number; y: number }>);
}

function autoLayout() {
  const pos: Record<number, NodePos> = {};
  ctrl.activeSteps.value.forEach((step, i) => {
    const col = i % NODES_PER_ROW;
    const row = Math.floor(i / NODES_PER_ROW);
    pos[step.id] = {
      x: col * (NODE_W + H_GAP),
      y: row * (NODE_H + V_GAP),
    };
  });
  nodePositions.value = pos;
  savePositions();
  void nextTick(() => updateArrows());
}

function ensureStepPosition(stepId: number) {
  if (nodePositions.value[stepId]) return;
  const steps = ctrl.activeSteps.value;
  const idx = steps.findIndex((s) => s.id === stepId);
  if (idx <= 0) {
    nodePositions.value[stepId] = { x: 0, y: 0 };
  } else {
    const prev = nodePositions.value[steps[idx - 1].id];
    if (prev) {
      nodePositions.value[stepId] = { x: prev.x + NODE_W + H_GAP, y: prev.y };
    } else {
      const col = idx % NODES_PER_ROW;
      const row = Math.floor(idx / NODES_PER_ROW);
      nodePositions.value[stepId] = { x: col * (NODE_W + H_GAP), y: row * (NODE_H + V_GAP) };
    }
  }
  savePositions();
}

const canvasSize = computed(() => {
  let maxX = 0;
  let maxY = 0;
  for (const pos of Object.values(nodePositions.value)) {
    maxX = Math.max(maxX, pos.x + NODE_W + 60);
    maxY = Math.max(maxY, pos.y + NODE_H + 60);
  }
  return { width: Math.max(maxX, 600), height: Math.max(maxY, 300) };
});

function startNodeDrag(stepId: number, event: MouseEvent) {
  if ((event.target as HTMLElement).closest("button")) return;
  event.preventDefault();
  draggingNodeId.value = stepId;
  nodeDragStartX = event.clientX;
  nodeDragStartY = event.clientY;
  const pos = nodePositions.value[stepId] ?? { x: 0, y: 0 };
  nodeDragOrigX = pos.x;
  nodeDragOrigY = pos.y;
  document.addEventListener("mousemove", onNodeDrag);
  document.addEventListener("mouseup", stopNodeDrag);
  document.body.style.userSelect = "none";
}

function onNodeDrag(event: MouseEvent) {
  if (draggingNodeId.value === null) return;
  const dx = event.clientX - nodeDragStartX;
  const dy = event.clientY - nodeDragStartY;
  nodePositions.value = {
    ...nodePositions.value,
    [draggingNodeId.value]: {
      x: Math.max(0, nodeDragOrigX + dx),
      y: Math.max(0, nodeDragOrigY + dy),
    },
  };
  updateArrows();
}

function stopNodeDrag() {
  draggingNodeId.value = null;
  document.removeEventListener("mousemove", onNodeDrag);
  document.removeEventListener("mouseup", stopNodeDrag);
  document.body.style.userSelect = "";
  savePositions();
  void nextTick(() => updateArrows());
}

const selectPt = {
  root: { class: "!bg-panel !border-divider" },
  label: { class: "!text-xs !py-1.5 !text-ink" },
  option: { class: "!text-xs" },
};
</script>

<template>
  <div class="flex flex-1 gap-3 overflow-hidden">
    <!-- Left sidebar: workflow list -->
    <aside
      v-if="!sidebarCollapsed"
      class="flex shrink-0 flex-col overflow-hidden rounded-lg border border-divider bg-panel shadow-sm"
      :style="{ width: sidebarWidth + 'px' }"
    >
      <div class="flex items-center gap-2 border-b border-divider p-3">
        <Button icon="pi pi-angle-double-left" text rounded size="small" title="Collapse sidebar" @click="toggleSidebar" />
        <span class="text-sm font-semibold text-ink">Workflows</span>
        <Button icon="pi pi-plus" size="small" class="ml-auto" title="New workflow" @click="openCreateWorkflowDialog" />
      </div>

      <div class="border-b border-divider px-3 py-2">
        <span class="flex items-center gap-2 rounded-md border border-divider bg-canvas px-2">
          <i class="pi pi-search text-xs text-muted" />
          <InputText
            v-model="ctrl.searchQuery.value"
            class="embedded-input w-full border-0 !bg-transparent !py-1.5 !text-xs"
            placeholder="Search workflows..."
          />
        </span>
      </div>

      <div class="flex-1 space-y-1 overflow-auto p-2">
        <div v-if="ctrl.isLoading.value" class="px-2 py-4 text-center text-xs text-muted">Loading...</div>

        <p v-else-if="ctrl.filteredWorkflows.value.length === 0" class="px-2 py-4 text-center text-xs text-muted">
          No workflows found.
        </p>

        <button
          v-for="wf in ctrl.filteredWorkflows.value"
          :key="wf.id"
          class="group flex w-full items-center gap-2 rounded-md px-2 py-2 text-left transition-colors"
          :class="wf.id === ctrl.activeId.value ? 'bg-brand/10 text-brand' : 'text-secondary hover:bg-canvas'"
          @click="ctrl.selectWorkflow(wf.id)"
        >
          <i class="pi pi-sitemap shrink-0 text-xs" />
          <span class="min-w-0 flex-1">
            <span class="block truncate text-xs font-bold">{{ wf.name }}</span>
            <span class="block truncate text-[10px] text-muted">{{ formatDate(wf.updatedAt) }} · {{ wf.stepCount }} steps</span>
          </span>
          <i
            class="pi pi-trash shrink-0 text-xs text-muted opacity-0 hover:text-red-500 group-hover:opacity-100"
            title="Delete workflow"
            @click.stop="confirmDeleteWorkflow"
          />
        </button>
      </div>
    </aside>

    <!-- Drag handle -->
    <div
      v-if="!sidebarCollapsed"
      class="group flex w-1.5 shrink-0 cursor-col-resize items-center justify-center"
      title="Drag to resize"
      @mousedown="startDrag"
    >
      <div class="h-10 w-1 rounded-full bg-divider transition-colors group-hover:bg-brand" />
    </div>

    <!-- Main area -->
    <div class="flex min-h-0 min-w-0 flex-1 flex-col gap-4 overflow-hidden">
      <!-- Expand sidebar button -->
      <Button
        v-if="sidebarCollapsed"
        icon="pi pi-angle-double-right"
        severity="secondary"
        outlined
        size="small"
        class="absolute left-3 top-3 z-10"
        title="Show sidebar"
        @click="toggleSidebar"
      />

      <!-- Empty state -->
      <div v-if="!ctrl.activeWorkflow.value" class="flex flex-1 items-center justify-center rounded-lg border border-dashed border-divider bg-panel/50 p-12">
        <div class="text-center">
          <i class="pi pi-sitemap text-4xl text-muted/60" />
          <p class="mt-2 text-sm text-muted">No workflow selected. Create one to get started.</p>
          <Button icon="pi pi-plus" label="Create Workflow" class="mt-4" @click="openCreateWorkflowDialog" />
        </div>
      </div>

      <template v-else>
        <!-- Header panel -->
        <div class="shrink-0 rounded-lg border border-divider bg-panel p-6 shadow-sm">
          <div class="flex flex-wrap items-center gap-3">
            <i class="pi pi-sitemap text-2xl text-muted" />
            <div class="min-w-0">
              <h2 class="text-lg font-semibold text-ink">{{ ctrl.activeWorkflow.value.name }}</h2>
              <p class="text-sm text-muted">{{ ctrl.activeWorkflow.value.description || "No description" }}</p>
            </div>
            <div class="ml-auto flex shrink-0 items-center gap-2">
              <Button icon="pi pi-objects-column" label="Auto layout" severity="secondary" size="small" title="Reset node positions" @click="autoLayout" />
              <Button icon="pi pi-pencil" label="Edit" severity="secondary" size="small" @click="openEditWorkflowDialog" />
              <Button icon="pi pi-copy" label="Duplicate" severity="secondary" size="small" @click="ctrl.duplicateWorkflow(ctrl.activeId.value!)" />
              <Button icon="pi pi-trash" label="Delete" severity="danger" text size="small" @click="confirmDeleteWorkflow" />
            </div>
          </div>
        </div>

        <!-- Diagram area -->
        <div class="relative flex-1 overflow-auto rounded-lg border border-divider bg-panel shadow-sm">
          <div
            ref="nodesContainer"
            class="relative"
            :style="{ minWidth: canvasSize.width + 'px', minHeight: canvasSize.height + 'px' }"
          >
            <!-- SVG arrow layer -->
            <svg class="pointer-events-none absolute inset-0 overflow-visible" :style="{ width: canvasSize.width + 'px', height: canvasSize.height + 'px' }" style="z-index: 0">
              <defs>
                <marker id="wf-arrowhead" markerWidth="10" markerHeight="7" refX="10" refY="3.5" orient="auto">
                  <polygon points="0 0, 10 3.5, 0 7" style="fill: rgb(var(--color-text-muted))" />
                </marker>
              </defs>
              <path
                v-for="arrow in arrowPaths"
                :key="arrow.id"
                :d="arrow.d"
                fill="none"
                stroke-width="2"
                stroke-linecap="round"
                marker-end="url(#wf-arrowhead)"
                style="stroke: rgb(var(--color-text-muted))"
              />
            </svg>

            <!-- Nodes (absolute positioned) -->
            <template v-for="(step, index) in ctrl.activeSteps.value" :key="step.id">
              <div
                :data-step-id="step.id"
                class="absolute z-10 w-52 rounded-lg border bg-panel p-4 shadow-card transition-shadow hover:shadow-float"
                :class="[
                  step.id === ctrl.selectedStepId.value ? 'border-brand ring-2 ring-brand/30' : 'border-divider',
                  draggingNodeId === step.id ? 'shadow-float cursor-grabbing opacity-80' : 'cursor-grab',
                ]"
                :style="{
                  left: (nodePositions[step.id]?.x ?? 0) + 'px',
                  top: (nodePositions[step.id]?.y ?? 0) + 'px',
                }"
                @mousedown="startNodeDrag(step.id, $event)"
                @click="ctrl.selectStep(step.id)"
                @dblclick="openEditStepDialog(step.id)"
              >
                <div class="mb-2 flex items-center gap-2">
                  <span class="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-brand/10 text-xs font-bold text-brand">
                    {{ index + 1 }}
                  </span>
                  <i :class="[step.icon, 'text-muted']" />
                  <h4 class="min-w-0 flex-1 break-words text-sm font-semibold text-ink">{{ step.name }}</h4>
                  <button
                    class="shrink-0 text-muted opacity-0 transition-opacity hover:text-red-500"
                    :class="{ 'opacity-100': step.id === ctrl.selectedStepId.value }"
                    title="Delete step"
                    @click.stop="confirmDeleteStep(step.id)"
                  >
                    <i class="pi pi-times text-[10px]" />
                  </button>
                </div>
                <span :class="['mt-1 inline-block rounded-full px-2 py-0.5 text-[11px] font-bold', stepTypeBadgeClass(step.type)]">
                  {{ stepTypeLabel(step.type) }}
                </span>
                <p class="mt-2 line-clamp-2 text-xs text-muted">{{ step.description }}</p>
              </div>
            </template>

            <!-- Add step button (fixed bottom-right of canvas) -->
            <button
              class="absolute z-10 flex h-10 items-center gap-2 rounded-lg border border-dashed border-divider px-4 text-sm text-muted transition-colors hover:border-brand hover:bg-brand/5 hover:text-brand"
              :style="{ left: '16px', top: (canvasSize.height - 20) + 'px' }"
              @click="openAddStepDialog(ctrl.activeSteps.value.length > 0 ? ctrl.activeSteps.value[ctrl.activeSteps.value.length - 1].id : null)"
            >
              <i class="pi pi-plus-circle" />
              <span>Add step</span>
            </button>
          </div>
        </div>

        <!-- Selected step detail panel -->
        <div v-if="ctrl.selectedStep.value" class="shrink-0 rounded-lg border border-divider bg-panel p-4 shadow-sm">
          <div class="flex items-center gap-3">
            <i :class="[ctrl.selectedStep.value.icon, 'text-xl text-brand']" />
            <div class="min-w-0 flex-1">
              <p class="truncate text-sm font-semibold text-ink">{{ ctrl.selectedStep.value.name }}</p>
              <p class="truncate text-xs text-muted">{{ ctrl.selectedStep.value.description || "No description" }}</p>
            </div>
            <span :class="['rounded-full px-2 py-0.5 text-[11px] font-bold', stepTypeBadgeClass(ctrl.selectedStep.value.type)]">
              {{ stepTypeLabel(ctrl.selectedStep.value.type) }}
            </span>
            <Button icon="pi pi-pencil" severity="secondary" text rounded size="small" title="Edit step" @click="openEditStepDialog(ctrl.selectedStep.value.id)" />
            <Button icon="pi pi-trash" severity="danger" text rounded size="small" title="Delete step" @click="confirmDeleteStep(ctrl.selectedStep.value.id)" />
            <Button icon="pi pi-times" severity="secondary" text rounded size="small" title="Close" @click="ctrl.selectStep(null)" />
          </div>
        </div>
      </template>
    </div>

    <!-- Add/Edit Workflow Dialog -->
    <Dialog
      :visible="showWorkflowDialog"
      class="w-full max-w-md rounded-lg bg-panel shadow-xl"
      :closable="true"
      modal
      @update:visible="showWorkflowDialog = $event"
    >
      <template #header>
        <h3 class="font-bold text-ink">{{ editingWorkflowId ? "Edit Workflow" : "New Workflow" }}</h3>
      </template>

      <div class="space-y-4">
        <label class="block">
          <span class="text-xs font-bold text-muted">Name <span class="text-red-500">*</span></span>
          <InputText v-model="wfName" class="mt-1 w-full" placeholder="e.g. Claude Feature Development" autofocus />
        </label>
        <label class="block">
          <span class="text-xs font-bold text-muted">Description</span>
          <InputText v-model="wfDescription" class="mt-1 w-full" placeholder="Brief description of this workflow" />
        </label>
      </div>

      <template #footer>
        <div class="flex items-center justify-end gap-2">
          <Button label="Cancel" severity="secondary" @click="showWorkflowDialog = false" />
          <Button :label="editingWorkflowId ? 'Save' : 'Create'" :disabled="!wfName.trim()" @click="saveWorkflow" />
        </div>
      </template>
    </Dialog>

    <!-- Add/Edit Step Dialog -->
    <Dialog
      :visible="showStepDialog"
      class="w-full max-w-md rounded-lg bg-panel shadow-xl"
      :closable="true"
      modal
      @update:visible="showStepDialog = $event"
    >
      <template #header>
        <h3 class="font-bold text-ink">{{ editingStepId ? "Edit Step" : "Add Step" }}</h3>
      </template>

      <div class="space-y-4">
        <div class="flex items-end gap-3">
          <label class="block min-w-0 flex-1">
            <span class="text-xs font-bold text-muted">Name <span class="text-red-500">*</span></span>
            <InputText v-model="stepName" class="mt-1 w-full" placeholder="e.g. Code Review" autofocus />
          </label>
          <label class="block">
            <span class="text-xs font-bold text-muted">Icon</span>
            <div class="mt-1 flex items-center gap-2">
              <div class="flex h-10 items-center gap-2 rounded-md border border-divider bg-panel px-3">
                <i :class="[stepIcon, 'text-brand']" />
                <InputText
                  v-model="stepIcon"
                  class="embedded-input w-24 border-0 !bg-transparent !p-0 !text-sm"
                  placeholder="pi pi-cog"
                />
              </div>
              <Button
                icon="pi pi-th-large"
                severity="secondary"
                outlined
                title="Browse icons"
                @click="showIconPicker = true"
              />
            </div>
          </label>
        </div>
        <label class="block">
          <span class="text-xs font-bold text-muted">Type</span>
          <Select
            v-model="stepType"
            :options="stepTypeOptions"
            optionLabel="label"
            optionValue="value"
            class="mt-1 w-full"
            :pt="selectPt"
            @change="onStepTypeChange"
          />
        </label>
        <label class="block">
          <span class="text-xs font-bold text-muted">Description</span>
          <InputText v-model="stepDescription" class="mt-1 w-full" placeholder="What this step does" />
        </label>
      </div>

      <template #footer>
        <div class="flex items-center justify-end gap-2">
          <Button label="Cancel" severity="secondary" @click="showStepDialog = false" />
          <Button :label="editingStepId ? 'Save' : 'Add'" :disabled="!stepName.trim()" @click="saveStep" />
        </div>
      </template>
    </Dialog>

    <!-- Icon Picker Dialog -->
    <IconPickerDialog
      :visible="showIconPicker"
      :selected="stepIcon"
      @update:visible="showIconPicker = $event"
      @select="(icon: string) => (stepIcon = 'pi ' + icon)"
    />

    <!-- Delete Confirmation Dialog -->
    <Dialog
      :visible="showDeleteDialog"
      class="w-full max-w-sm rounded-lg bg-panel shadow-xl"
      :closable="true"
      modal
      @update:visible="showDeleteDialog = $event"
    >
      <template #header>
        <h3 class="font-bold text-ink">Confirm Delete</h3>
      </template>

      <p class="text-sm text-ink">
        Are you sure you want to delete
        <span class="font-semibold">{{ deleteTarget?.name }}</span>?
        This action cannot be undone.
      </p>

      <template #footer>
        <div class="flex items-center justify-end gap-2">
          <Button label="Cancel" severity="secondary" @click="showDeleteDialog = false" />
          <Button label="Delete" severity="danger" @click="executeDelete" />
        </div>
      </template>
    </Dialog>
  </div>
</template>
