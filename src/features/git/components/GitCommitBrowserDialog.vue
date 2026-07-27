<script setup lang="ts">
import { ref, watch } from "vue";
import Button from "primevue/button";
import Checkbox from "primevue/checkbox";
import Dialog from "primevue/dialog";
import type { GitApi } from "../composables/useGit";
import { statusMeta, baseName } from "../utils/fileStatus";
import type { GitCommit, GitFileChange } from "@/_/types/git";

const props = defineProps<{
  git: GitApi;
  onFileContext: (e: MouseEvent, rel: string) => void;
}>();
const visible = defineModel<boolean>("visible", { default: false });

const browserSelected = ref<Set<string>>(new Set());
const browserFocusedHash = ref("");
const browserFileSel = ref("");

watch(visible, async (v) => {
  if (!v) return;
  browserSelected.value = new Set();
  browserFocusedHash.value = "";
  browserFileSel.value = "";
  await props.git.loadBrowserCommits();
  if (props.git.browserCommits.value.length) focusBrowser(props.git.browserCommits.value[0]);
});

function focusBrowser(c: GitCommit) {
  browserFocusedHash.value = c.hash;
  browserFileSel.value = "";
  void props.git.focusBrowserCommit(c.hash);
}
function selectBrowserFile(f: GitFileChange) {
  browserFileSel.value = f.path;
  void props.git.selectBrowserFile(browserFocusedHash.value, f.path);
}
function toggleBrowserSel(hash: string) {
  const s = new Set(browserSelected.value);
  if (s.has(hash)) s.delete(hash);
  else s.add(hash);
  browserSelected.value = s;
}
function copySelectedShas() {
  const shas = props.git.browserCommits.value
    .filter((c) => browserSelected.value.has(c.hash))
    .map((c) => c.hash);
  if (shas.length) props.git.copyText(shas.join("\n"), `${shas.length} SHA`);
}
</script>

<template>
  <Dialog v-model:visible="visible" modal header="Duyệt commit" :style="{ width: '900px' }">
    <div class="flex h-[460px] gap-2">
      <!-- commit list (multi-select) -->
      <div class="flex w-80 shrink-0 flex-col overflow-hidden rounded-md border border-divider">
        <div class="flex items-center gap-2 border-b border-divider bg-canvas px-2 py-1 text-[11px] text-muted">
          <span class="font-bold uppercase tracking-wide">Commits</span>
          <span v-if="browserSelected.size" class="ml-auto text-brand">{{ browserSelected.size }} chọn</span>
        </div>
        <div class="min-h-0 flex-1 overflow-y-auto">
          <div v-if="git.browserLoading.value" class="p-6 text-center text-sm text-muted">
            <i class="pi pi-spinner pi-spin mr-1.5" /> Đang tải…
          </div>
          <div
            v-for="c in git.browserCommits.value"
            v-else
            :key="c.hash"
            class="flex items-start gap-2 border-b border-divider-light px-2 py-1.5 transition-colors hover:bg-canvas"
            :class="browserFocusedHash === c.hash ? 'bg-canvas' : ''"
          >
            <Checkbox
              :model-value="browserSelected.has(c.hash)"
              binary
              class="mt-0.5"
              @click.stop
              @change="toggleBrowserSel(c.hash)"
            />
            <button class="min-w-0 flex-1 text-left" @click="focusBrowser(c)">
              <span class="block truncate text-sm text-ink">{{ c.subject }}</span>
              <span class="flex items-center gap-2 text-[10px] text-muted">
                <span class="font-mono">{{ c.short_hash }}</span>
                <span class="truncate">{{ c.author_name }}</span>
                <span class="ml-auto shrink-0">{{ c.relative_date }}</span>
              </span>
            </button>
          </div>
        </div>
      </div>
      <!-- files of focused commit -->
      <div class="flex w-56 shrink-0 flex-col overflow-hidden rounded-md border border-divider">
        <div class="border-b border-divider bg-canvas px-2 py-1 text-[11px] font-bold uppercase tracking-wide text-muted">
          Files ({{ git.browserFiles.value.length }})
        </div>
        <div class="min-h-0 flex-1 overflow-y-auto">
          <button
            v-for="f in git.browserFiles.value"
            :key="f.path"
            class="flex w-full items-center gap-2 px-2 py-1 text-left transition-colors hover:bg-canvas"
            :class="browserFileSel === f.path ? 'bg-canvas' : ''"
            @click="selectBrowserFile(f)"
            @contextmenu="onFileContext($event, f.path)"
          >
            <span class="shrink-0 text-xs font-bold" :class="statusMeta(f.status).cls">{{ f.status }}</span>
            <span class="min-w-0 flex-1 truncate text-xs text-ink" :title="f.path">{{ baseName(f.path) }}</span>
          </button>
          <div v-if="!git.browserFiles.value.length" class="p-4 text-center text-xs text-muted">—</div>
        </div>
      </div>
      <!-- diff -->
      <div class="min-h-0 flex-1 overflow-auto rounded-md border border-divider">
        <div v-if="!git.browserDiff.value" class="flex h-full items-center justify-center p-6 text-center text-xs text-muted">
          Chọn một file để xem diff.
        </div>
        <div v-else-if="git.browserDiff.value.is_binary" class="p-4 text-xs text-muted">File nhị phân.</div>
        <table v-else class="w-full border-collapse font-mono text-xs leading-5">
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
              <td class="w-10 select-none border-r border-divider px-2 text-right text-[10px] text-muted">{{ line.old_line || "" }}</td>
              <td class="w-10 select-none border-r border-divider px-2 text-right text-[10px] text-muted">{{ line.new_line || "" }}</td>
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
    <template #footer>
      <span class="mr-auto text-xs text-muted">Tick chọn nhiều commit rồi copy SHA.</span>
      <Button size="small" outlined severity="secondary" @click="visible = false">Đóng</Button>
      <Button size="small" :disabled="!browserSelected.size" @click="copySelectedShas">
        <i class="pi pi-copy mr-1.5" /> Copy {{ browserSelected.size }} SHA
      </Button>
    </template>
  </Dialog>
</template>
