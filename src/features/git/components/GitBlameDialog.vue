<script setup lang="ts">
import { computed, watch } from "vue";
import Button from "primevue/button";
import Dialog from "primevue/dialog";
import type { GitApi } from "../composables/useGit";
import { baseName } from "../utils/fileStatus";
import type { GitBlameLine } from "@/_/types/git";

const props = defineProps<{
  git: GitApi;
  file: string;
}>();
const emit = defineEmits<{
  "open-in-history": [hash: string];
}>();
const visible = defineModel<boolean>("visible", { default: false });

watch(visible, async (v) => {
  if (!v || !props.file) return;
  await props.git.loadBlame(props.file);
});

function isUncommitted(hash: string) {
  return /^0+$/.test(hash);
}

function isGroupStart(lines: GitBlameLine[], i: number) {
  return i === 0 || lines[i - 1].hash !== lines[i].hash;
}

const rows = computed(() =>
  props.git.blameLines.value.map((line, i) => ({
    line,
    groupStart: isGroupStart(props.git.blameLines.value, i),
  })),
);

function selectLine(line: GitBlameLine) {
  if (isUncommitted(line.hash)) return;
  void props.git.selectBlameLine(line.hash);
}

function copySha() {
  const hash = props.git.blameSelectedHash.value;
  if (hash) void props.git.copyText(hash, "SHA");
}

function openInHistory() {
  const hash = props.git.blameSelectedHash.value;
  if (!hash) return;
  emit("open-in-history", hash);
  visible.value = false;
}
</script>

<template>
  <Dialog
    v-model:visible="visible"
    modal
    maximizable
    header="Git blame"
    :style="{ width: '95vw', maxWidth: '1400px' }"
  >
    <div class="flex h-[70vh] max-h-[80vh] min-h-[420px] gap-2">
      <!-- blame lines -->
      <div class="flex min-w-0 flex-1 flex-col overflow-hidden rounded-md border border-divider">
        <div class="flex items-center gap-2 border-b border-divider bg-canvas px-2 py-1 text-[11px] text-muted">
          <i class="pi pi-file text-[10px]" />
          <span class="truncate font-mono" :title="file">{{ baseName(file) }}</span>
          <span v-if="git.blameLines.value.length" class="ml-auto shrink-0">{{ git.blameLines.value.length }} dòng</span>
        </div>
        <div class="min-h-0 flex-1 overflow-auto">
          <div v-if="git.blameLoading.value" class="p-6 text-center text-sm text-muted">
            <i class="pi pi-spinner pi-spin mr-1.5" /> Đang phân tích blame…
          </div>
          <div v-else-if="!rows.length" class="p-6 text-center text-sm text-muted">
            Không có dữ liệu blame.
          </div>
          <button
            v-for="(row, i) in rows"
            v-else
            :key="i"
            class="grid w-full grid-cols-[52px_200px_1fr] items-start gap-2 px-2 py-0.5 text-left font-mono text-xs transition-colors hover:bg-canvas"
            :class="[
              git.blameSelectedHash.value === row.line.hash && !isUncommitted(row.line.hash) ? 'bg-brand/10' : '',
              row.groupStart ? 'border-t border-divider-light' : '',
            ]"
            @click="selectLine(row.line)"
          >
            <span class="select-none text-right text-muted">{{ row.line.line_no }}</span>
            <span v-if="row.groupStart" class="truncate">
              <template v-if="isUncommitted(row.line.hash)">
                <span class="italic text-muted">Chưa commit</span>
              </template>
              <template v-else>
                <span class="font-semibold text-ink">{{ row.line.author_name || "Không rõ" }}</span>
                <span class="ml-1 text-muted">· {{ row.line.relative_date || row.line.short_hash }}</span>
              </template>
            </span>
            <span v-else class="select-none text-muted">·</span>
            <span class="whitespace-pre-wrap break-all text-secondary">{{ row.line.content }}</span>
          </button>
        </div>
      </div>

      <!-- commit detail của dòng đang chọn -->
      <div class="flex w-72 shrink-0 flex-col overflow-hidden rounded-md border border-divider">
        <div class="border-b border-divider bg-canvas px-2 py-1 text-[11px] font-bold uppercase tracking-wide text-muted">
          Chi tiết commit
        </div>
        <div class="min-h-0 flex-1 overflow-y-auto p-3">
          <div v-if="!git.blameDetail.value" class="text-center text-xs text-muted">
            Chọn một dòng để xem chi tiết commit.
          </div>
          <div v-else class="flex flex-col gap-2 text-sm">
            <p class="font-semibold text-ink">{{ git.blameDetail.value.commit.subject }}</p>
            <p v-if="git.blameDetail.value.body" class="whitespace-pre-wrap text-xs text-secondary">
              {{ git.blameDetail.value.body }}
            </p>
            <div class="mt-1 flex flex-col gap-1 text-[11px] text-muted">
              <span>{{ git.blameDetail.value.commit.author_name }} &lt;{{ git.blameDetail.value.commit.author_email }}&gt;</span>
              <span>{{ git.blameDetail.value.commit.relative_date }}</span>
              <span class="font-mono">{{ git.blameDetail.value.commit.short_hash }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
    <template #footer>
      <span class="mr-auto text-xs text-muted">Bấm vào một dòng để xem commit đã sửa dòng đó.</span>
      <Button size="small" outlined :disabled="!git.blameSelectedHash.value" @click="copySha">
        <i class="pi pi-copy mr-1.5" /> Copy SHA
      </Button>
      <Button size="small" outlined :disabled="!git.blameSelectedHash.value" @click="openInHistory">
        <i class="pi pi-history mr-1.5" /> Xem trong History
      </Button>
      <Button size="small" outlined severity="secondary" @click="visible = false">Đóng</Button>
    </template>
  </Dialog>
</template>
