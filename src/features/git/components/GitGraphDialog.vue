<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from "vue";
import Button from "primevue/button";
import Dialog from "primevue/dialog";
import Select from "primevue/select";
import type { GitApi } from "../composables/useGit";
import { statusMeta, baseName } from "../utils/fileStatus";
import type { GitFileChange, GitGraphCommit } from "@/_/types/git";

const props = defineProps<{
  git: GitApi;
  onFileContext: (e: MouseEvent, rel: string) => void;
}>();
const visible = defineModel<boolean>("visible", { default: false });

const GRAPH_ROW_H = 32;
const GRAPH_LANE_GAP = 16;
const GRAPH_PAD_X = 14;
const GRAPH_COLORS = [
  "#0d9373", "#3b82f6", "#a855f7", "#f59e0b",
  "#ef4444", "#06b6d4", "#ec4899", "#84cc16",
];
const graphLimit = ref(300);
const graphLimitOptions = [
  { label: "100", value: 100 },
  { label: "300", value: 300 },
  { label: "500", value: 500 },
  { label: "1000", value: 1000 },
  { label: "2000", value: 2000 },
];
const graphSelected = ref<GitGraphCommit | null>(null);
const graphFileSel = ref("");
const isMaximized = ref(false);

// === Drag-to-resize panel chi tiết commit (nhớ độ rộng vào localStorage) ===
const DETAIL_WIDTH_KEY = "git.width.graphDetail";
function loadWidth(key: string, def: number, min: number, max: number) {
  const raw = Number(localStorage.getItem(key) ?? "");
  return Number.isFinite(raw) && raw > 0 ? Math.max(min, Math.min(max, raw)) : def;
}
const detailWidth = ref(loadWidth(DETAIL_WIDTH_KEY, 440, 300, 720));
const rowRef = ref<HTMLElement | null>(null);
const isResizing = ref(false);
let activeMove: ((e: MouseEvent) => void) | null = null;

function endResize() {
  isResizing.value = false;
  if (activeMove) document.removeEventListener("mousemove", activeMove);
  document.removeEventListener("mouseup", endResize);
  activeMove = null;
  localStorage.setItem(DETAIL_WIDTH_KEY, String(Math.round(detailWidth.value)));
}

function startResizeDetail(e: MouseEvent) {
  e.preventDefault();
  isResizing.value = true;
  activeMove = (ev) => {
    const right = rowRef.value?.getBoundingClientRect().right ?? 0;
    detailWidth.value = Math.max(300, Math.min(720, right - ev.clientX));
  };
  document.addEventListener("mousemove", activeMove);
  document.addEventListener("mouseup", endResize);
}

onUnmounted(() => {
  if (activeMove) document.removeEventListener("mousemove", activeMove);
  document.removeEventListener("mouseup", endResize);
});

watch(visible, (v) => {
  if (v) {
    graphSelected.value = null;
    graphFileSel.value = "";
    isMaximized.value = false;
    props.git.loadGraph(graphLimit.value);
  }
});

function reloadGraph() {
  props.git.loadGraph(graphLimit.value);
}
/** Click node/row → xem chi tiết + danh sách file của commit (tái dùng loader). */
function selectGraphCommit(c: GitGraphCommit) {
  graphSelected.value = c;
  graphFileSel.value = "";
  void props.git.focusBrowserCommit(c.hash);
}
function onGraphFile(f: GitFileChange) {
  if (!graphSelected.value) return;
  graphFileSel.value = f.path;
  void props.git.selectBrowserFile(graphSelected.value.hash, f.path);
}

/** Tính lane/column cho từng commit rồi dựng node + edge (đường cong) cho SVG. */
const graphLayout = computed(() => {
  const commits = props.git.graphCommits.value;
  const n = commits.length;
  const rowOf = new Map<string, number>();
  commits.forEach((c, i) => rowOf.set(c.hash, i));

  const lanes: (string | null)[] = [];
  const colOf: number[] = new Array(n).fill(0);
  const firstFree = () => {
    const idx = lanes.indexOf(null);
    if (idx !== -1) return idx;
    lanes.push(null);
    return lanes.length - 1;
  };

  for (let i = 0; i < n; i++) {
    const c = commits[i];
    let col = lanes.indexOf(c.hash);
    if (col === -1) col = firstFree();
    colOf[i] = col;
    for (let j = 0; j < lanes.length; j++) {
      if (j !== col && lanes[j] === c.hash) lanes[j] = null;
    }
    if (!c.parents.length) {
      lanes[col] = null;
    } else {
      lanes[col] = c.parents[0];
      for (let k = 1; k < c.parents.length; k++) {
        const p = c.parents[k];
        if (lanes.indexOf(p) === -1) lanes[firstFree()] = p;
      }
    }
  }

  const colX = (col: number) => GRAPH_PAD_X + col * GRAPH_LANE_GAP;
  const rowY = (row: number) => row * GRAPH_ROW_H + GRAPH_ROW_H / 2;
  const colorAt = (col: number) => GRAPH_COLORS[col % GRAPH_COLORS.length];

  let maxCol = 0;
  for (let i = 0; i < n; i++) maxCol = Math.max(maxCol, colOf[i]);

  const nodes = commits.map((c, i) => ({
    x: colX(colOf[i]),
    y: rowY(i),
    color: colorAt(colOf[i]),
  }));

  const edges: { d: string; color: string }[] = [];
  for (let i = 0; i < n; i++) {
    const c = commits[i];
    const x1 = colX(colOf[i]);
    const y1 = rowY(i);
    for (const p of c.parents) {
      const pr = rowOf.get(p);
      if (pr === undefined) {
        edges.push({ d: `M ${x1} ${y1} L ${x1} ${y1 + GRAPH_ROW_H * 0.6}`, color: colorAt(colOf[i]) });
        continue;
      }
      const x2 = colX(colOf[pr]);
      const y2 = rowY(pr);
      const color = colorAt(colOf[pr]);
      const d =
        x1 === x2
          ? `M ${x1} ${y1} L ${x2} ${y2}`
          : `M ${x1} ${y1} C ${x1} ${(y1 + y2) / 2}, ${x2} ${(y1 + y2) / 2}, ${x2} ${y2}`;
      edges.push({ d, color });
    }
  }

  return {
    nodes,
    edges,
    width: colX(maxCol) + GRAPH_PAD_X,
    height: Math.max(n * GRAPH_ROW_H, GRAPH_ROW_H),
    rowH: GRAPH_ROW_H,
  };
});

function refLabel(r: string) {
  return r.replace("HEAD -> ", "").replace("tag: ", "");
}
function refClass(r: string) {
  if (r.startsWith("HEAD")) return "badge-success";
  if (r.startsWith("tag:")) return "badge-warning";
  if (r.includes("/")) return "badge-info";
  return "badge-neutral";
}
</script>

<template>
  <Dialog
    v-model:visible="visible"
    modal
    maximizable
    header="Visualization — đồ thị commit"
    :style="{ width: '1100px' }"
    @maximize="isMaximized = true"
    @unmaximize="isMaximized = false"
  >
    <div class="flex flex-col gap-2">
      <!-- Thanh điều khiển -->
      <div class="flex items-center gap-2">
        <span class="text-xs text-muted">{{ git.graphCommits.value.length }} commit (tất cả branch)</span>
        <div class="ml-auto flex items-center gap-2">
          <span class="text-xs text-muted">Số lượng:</span>
          <Select
            v-model="graphLimit"
            :options="graphLimitOptions"
            option-label="label"
            option-value="value"
            class="w-24"
            @change="reloadGraph"
          />
          <button
            class="rounded p-1.5 text-muted transition-colors hover:bg-canvas hover:text-brand"
            title="Làm mới"
            @click="reloadGraph"
          >
            <i class="pi text-xs" :class="git.graphLoading.value ? 'pi-spinner pi-spin' : 'pi-refresh'" />
          </button>
        </div>
      </div>

      <div
        ref="rowRef"
        class="flex gap-2"
        :class="[isMaximized ? 'h-[calc(100vh-230px)]' : 'h-[520px]', isResizing ? 'select-none' : '']"
      >
        <!-- Graph + rows -->
        <div class="min-w-0 flex-1 overflow-auto rounded-md border border-divider">
          <div v-if="git.graphLoading.value" class="p-8 text-center text-sm text-muted">
            <i class="pi pi-spinner pi-spin mr-1.5" /> Đang tải…
          </div>
          <div v-else-if="!git.graphCommits.value.length" class="p-8 text-center text-sm text-muted">
            Không có commit.
          </div>
          <div v-else class="relative" :style="{ height: graphLayout.height + 'px' }">
            <svg :width="graphLayout.width" :height="graphLayout.height" class="pointer-events-none absolute left-0 top-0">
              <path
                v-for="(e, i) in graphLayout.edges"
                :key="'e' + i"
                :d="e.d"
                :stroke="e.color"
                fill="none"
                stroke-width="2"
                stroke-linecap="round"
              />
              <circle
                v-for="(nd, i) in graphLayout.nodes"
                :key="'n' + i"
                :cx="nd.x"
                :cy="nd.y"
                r="4.5"
                :fill="nd.color"
                stroke-width="2"
                :style="{ stroke: 'rgb(var(--color-panel))' }"
              />
            </svg>
            <button
              v-for="(c, i) in git.graphCommits.value"
              :key="c.hash"
              class="absolute flex items-center gap-2 pr-3 text-left transition-colors hover:bg-canvas"
              :class="graphSelected?.hash === c.hash ? 'bg-canvas' : ''"
              :style="{ top: i * graphLayout.rowH + 'px', left: graphLayout.width + 'px', right: '0', height: graphLayout.rowH + 'px' }"
              @click="selectGraphCommit(c)"
            >
              <span
                v-for="r in c.refs"
                :key="r"
                class="shrink-0"
                :class="refClass(r)"
              >
                {{ refLabel(r) }}
              </span>
              <span class="min-w-0 flex-1 truncate text-sm text-ink">{{ c.subject }}</span>
              <span class="w-24 shrink-0 truncate text-[11px] text-muted">{{ c.author_name }}</span>
              <span class="w-20 shrink-0 truncate text-right text-[11px] text-muted">{{ c.relative_date }}</span>
              <span class="w-16 shrink-0 font-mono text-[11px] text-muted">{{ c.short_hash }}</span>
            </button>
          </div>
        </div>

        <!-- Resize handle: graph | chi tiết commit -->
        <div
          v-if="graphSelected"
          class="flex w-2 shrink-0 cursor-col-resize items-center justify-center hover:bg-brand/10"
          :class="isResizing ? 'bg-brand/20' : ''"
          @mousedown="startResizeDetail"
        >
          <div class="h-8 w-0.5 rounded-full bg-divider" :class="isResizing ? 'bg-brand' : ''" />
        </div>

        <!-- Chi tiết commit đã chọn -->
        <div
          v-if="graphSelected"
          class="flex shrink-0 flex-col overflow-hidden rounded-md border border-divider"
          :style="{ width: detailWidth + 'px' }"
        >
          <div class="border-b border-divider bg-canvas px-3 py-2">
            <p class="text-sm font-semibold text-ink">{{ graphSelected.subject }}</p>
            <p class="mt-0.5 flex flex-wrap items-center gap-2 text-[11px] text-muted">
              <span>{{ graphSelected.author_name }}</span>
              <span>·</span>
              <span>{{ graphSelected.relative_date }}</span>
              <span class="font-mono">{{ graphSelected.short_hash }}</span>
              <button class="rounded p-0.5 hover:text-brand" title="Copy SHA" @click="git.copyText(graphSelected.hash, 'SHA')">
                <i class="pi pi-copy text-[10px]" />
              </button>
            </p>
          </div>
          <div class="border-b border-divider bg-canvas px-2 py-1 text-[11px] font-bold uppercase tracking-wide text-muted">
            Files ({{ git.browserFiles.value.length }})
          </div>
          <div class="max-h-40 shrink-0 overflow-y-auto border-b border-divider">
            <button
              v-for="f in git.browserFiles.value"
              :key="f.path"
              class="flex w-full items-center gap-2 px-2 py-1 text-left transition-colors hover:bg-canvas"
              :class="graphFileSel === f.path ? 'bg-canvas' : ''"
              @click="onGraphFile(f)"
              @contextmenu="onFileContext($event, f.path)"
            >
              <span class="shrink-0 text-xs font-bold" :class="statusMeta(f.status).cls">{{ f.status }}</span>
              <span class="min-w-0 flex-1 truncate text-xs text-ink" :title="f.path">{{ baseName(f.path) }}</span>
            </button>
            <div v-if="!git.browserFiles.value.length" class="p-3 text-center text-xs text-muted">—</div>
          </div>
          <div class="min-h-0 flex-1 overflow-auto">
            <div v-if="!git.browserDiff.value" class="flex h-full items-center justify-center p-4 text-center text-xs text-muted">
              Chọn một file để xem diff.
            </div>
            <div v-else-if="git.browserDiff.value.is_binary" class="p-3 text-xs text-muted">File nhị phân.</div>
            <table v-else class="w-full border-collapse font-mono text-[11px] leading-5">
              <tbody>
                <tr
                  v-for="(line, i) in git.browserDiff.value.lines"
                  :key="i"
                  :class="{
                    'bg-emerald-50': line.kind === 'add',
                    'bg-red-50': line.kind === 'del',
                    'bg-slate-100': line.kind === 'hunk',
                  }"
                >
                  <td class="w-8 select-none border-r border-divider px-1 text-right text-[10px] text-muted">{{ line.old_line || "" }}</td>
                  <td class="w-8 select-none border-r border-divider px-1 text-right text-[10px] text-muted">{{ line.new_line || "" }}</td>
                  <td
                    class="whitespace-pre-wrap break-all px-2"
                    :class="{
                      'text-emerald-700': line.kind === 'add',
                      'text-red-700': line.kind === 'del',
                      'font-semibold text-sky-700': line.kind === 'hunk',
                      'text-secondary': line.kind === 'context',
                    }"
                  ><span class="select-none text-muted">{{ line.kind === 'add' ? '+' : line.kind === 'del' ? '-' : ' ' }}</span>{{ line.content }}</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </div>
    </div>
    <template #footer>
      <Button size="small" outlined severity="secondary" @click="visible = false">Đóng</Button>
    </template>
  </Dialog>
</template>
