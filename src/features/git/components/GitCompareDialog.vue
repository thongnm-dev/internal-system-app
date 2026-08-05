<script setup lang="ts">
import { computed, ref, watch } from "vue";
import Dialog from "primevue/dialog";
import Select from "primevue/select";
import DialogFooter from "@/shared/components/DialogFooter.vue";
import type { GitApi } from "../composables/useGit";
import { guessBase, resolveRef } from "../utils/gitRefs";
import { statusMeta, baseName } from "../utils/fileStatus";

const props = defineProps<{
  git: GitApi;
  pr: { base: string; head: string } | null;
  onFileContext: (e: MouseEvent, rel: string) => void;
}>();
const visible = defineModel<boolean>("visible", { default: false });

const cmpBase = ref("");
const cmpHead = ref("");
const isMaximized = ref(false);

const allBranchRefs = computed(() =>
  props.git.branches.value
    .filter((b) => !b.name.endsWith("/HEAD"))
    .map((b) => ({ label: b.is_remote ? `${b.name} (remote)` : b.name, value: b.name })),
);

watch(visible, (v) => {
  if (!v) return;
  isMaximized.value = false;
  if (props.pr) {
    cmpBase.value = resolveRef(props.git.branches.value, props.pr.base);
    cmpHead.value = resolveRef(props.git.branches.value, props.pr.head);
  } else {
    cmpHead.value = props.git.info.value?.current_branch || "";
    cmpBase.value = guessBase(
      props.git.branches.value.map((b) => b.name),
      cmpHead.value,
      props.git.info.value?.upstream,
    );
  }
  props.git.comparison.value = null;
  props.git.comparisonDiff.value = null;
  void runCompare();
});

async function runCompare() {
  if (cmpBase.value && cmpHead.value && cmpBase.value !== cmpHead.value) {
    await props.git.compareBranches(cmpBase.value, cmpHead.value);
  }
}

async function doCreatePR() {
  await props.git.createPullRequest(cmpBase.value, cmpHead.value);
}
</script>

<template>
  <Dialog
    v-model:visible="visible"
    modal
    maximizable
    header="So sánh branch / Pull Request"
    :style="{ width: '840px' }"
    @maximize="isMaximized = true"
    @unmaximize="isMaximized = false"
  >
    <div class="flex flex-col gap-3">
      <div class="flex items-end gap-2">
        <div class="min-w-0 flex-1">
          <label class="mb-1 block text-xs font-medium text-muted">Base (đích merge vào)</label>
          <Select
            v-model="cmpBase"
            :options="allBranchRefs"
            option-label="label"
            option-value="value"
            filter
            class="w-full"
            @change="runCompare"
          />
        </div>
        <i class="pi pi-arrow-left mb-2 shrink-0 text-muted" />
        <div class="min-w-0 flex-1">
          <label class="mb-1 block text-xs font-medium text-muted">Head (nguồn)</label>
          <Select
            v-model="cmpHead"
            :options="allBranchRefs"
            option-label="label"
            option-value="value"
            filter
            class="w-full"
            @change="runCompare"
          />
        </div>
      </div>

      <p v-if="cmpBase === cmpHead" class="text-xs text-amber-600">
        Base và head đang trùng nhau — hãy chọn hai branch khác nhau.
      </p>

      <div v-if="git.comparison.value" class="flex flex-wrap items-center gap-2 text-xs">
        <span class="badge-success">
          {{ git.comparison.value.ahead }} commit sẽ vào PR
        </span>
        <span v-if="git.comparison.value.behind" class="badge-info">
          base đi trước {{ git.comparison.value.behind }}
        </span>
        <span class="text-muted">{{ git.comparison.value.files.length }} file thay đổi</span>
      </div>

      <div
        v-if="git.comparison.value"
        class="flex gap-2"
        :class="isMaximized ? 'h-[calc(100vh-300px)]' : 'h-[380px]'"
      >
        <!-- commits + files -->
        <div class="flex w-64 shrink-0 flex-col overflow-hidden rounded-md border border-divider">
          <div class="border-b border-divider bg-canvas px-2 py-1 text-[11px] font-bold uppercase tracking-wide text-muted">
            Commits ({{ git.comparison.value.commits.length }})
          </div>
          <div class="max-h-36 overflow-y-auto">
            <div
              v-for="c in git.comparison.value.commits"
              :key="c.hash"
              class="border-b border-divider-light px-2 py-1"
            >
              <p class="truncate text-xs text-ink">{{ c.subject }}</p>
              <p class="text-[10px] text-muted">{{ c.author_name }} · {{ c.short_hash }}</p>
            </div>
            <div v-if="!git.comparison.value.commits.length" class="p-3 text-center text-xs text-muted">
              Không có commit chênh lệch.
            </div>
          </div>
          <div class="border-y border-divider bg-canvas px-2 py-1 text-[11px] font-bold uppercase tracking-wide text-muted">
            Files ({{ git.comparison.value.files.length }})
          </div>
          <div class="min-h-0 flex-1 overflow-y-auto">
            <button
              v-for="f in git.comparison.value.files"
              :key="f.path"
              class="flex w-full items-center gap-2 px-2 py-1 text-left transition-colors hover:bg-canvas"
              :class="git.comparisonDiff.value?.path === f.path ? 'bg-canvas' : ''"
              @click="git.compareSelectFile(f)"
              @contextmenu="onFileContext($event, f.path)"
            >
              <span class="shrink-0 text-xs font-bold" :class="statusMeta(f.status).cls">{{ f.status }}</span>
              <span class="min-w-0 flex-1 truncate text-xs text-ink" :title="f.path">{{ baseName(f.path) }}</span>
            </button>
          </div>
        </div>
        <!-- diff -->
        <div class="min-h-0 flex-1 overflow-auto rounded-md border border-divider">
          <div v-if="!git.comparisonDiff.value" class="flex h-full items-center justify-center p-6 text-center text-xs text-muted">
            Chọn một file để xem diff.
          </div>
          <div v-else-if="git.comparisonDiff.value.is_binary" class="p-4 text-xs text-muted">
            File nhị phân — không hiển thị diff.
          </div>
          <table v-else class="w-full border-collapse font-mono text-xs leading-5">
            <tbody>
              <tr
                v-for="(line, i) in git.comparisonDiff.value.lines"
                :key="i"
                :class="{
                  'bg-emerald-50': line.kind === 'add',
                  'bg-red-50': line.kind === 'del',
                  'bg-slate-100': line.kind === 'hunk',
                }"
              >
                <td class="w-10 select-none border-r border-divider px-2 text-right text-[10px] text-muted">
                  {{ line.old_line || "" }}
                </td>
                <td class="w-10 select-none border-r border-divider px-2 text-right text-[10px] text-muted">
                  {{ line.new_line || "" }}
                </td>
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
    <template #footer>
      <DialogFooter
        cancel-label="Đóng"
        confirm-label="Tạo Pull Request"
        confirm-icon="pi pi-external-link"
        :confirm-disabled="!git.comparison.value?.web_url || !git.comparison.value?.ahead"
        @cancel="visible = false"
        @confirm="doCreatePR"
      >
        <template #extra>
          <span
            v-if="git.comparison.value && !git.comparison.value.web_url"
            class="mr-auto text-xs text-amber-600"
          >
            Repo không có remote origin — không tạo được Pull Request.
          </span>
        </template>
      </DialogFooter>
    </template>
  </Dialog>
</template>
