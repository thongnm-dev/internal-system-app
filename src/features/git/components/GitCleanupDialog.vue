<script setup lang="ts">
import { ref, watch } from "vue";
import Button from "primevue/button";
import Checkbox from "primevue/checkbox";
import Dialog from "primevue/dialog";
import type { GitApi } from "../composables/useGit";

const props = defineProps<{ git: GitApi }>();
const visible = defineModel<boolean>("visible", { default: false });

const cleanupList = ref<string[]>([]);
const cleanupSelected = ref<Set<string>>(new Set());
const cleanupScanning = ref(false);

watch(visible, async (v) => {
  if (!v) return;
  cleanupScanning.value = true;
  cleanupList.value = [];
  cleanupSelected.value = new Set();
  cleanupList.value = await props.git.cleanupScan();
  cleanupSelected.value = new Set(cleanupList.value);
  cleanupScanning.value = false;
});

function toggleCleanup(name: string) {
  const s = new Set(cleanupSelected.value);
  if (s.has(name)) s.delete(name);
  else s.add(name);
  cleanupSelected.value = s;
}

async function doCleanup() {
  await props.git.cleanupDelete([...cleanupSelected.value]);
  visible.value = false;
}
</script>

<template>
  <Dialog v-model:visible="visible" modal header="Cleanup branch đã merge" :style="{ width: '520px' }">
    <div class="flex flex-col gap-2">
      <p class="text-xs text-muted">
        Đã <strong>fetch --prune</strong>. Các branch local có remote đã bị xóa (thường sau khi PR đã merge &amp; xóa nhánh):
      </p>
      <div v-if="cleanupScanning" class="p-6 text-center text-sm text-muted">
        <i class="pi pi-spinner pi-spin mr-1.5" /> Đang quét…
      </div>
      <div v-else-if="!cleanupList.length" class="p-6 text-center text-sm text-muted">
        Không có branch nào cần dọn. 🎉
      </div>
      <div v-else class="max-h-64 overflow-y-auto rounded-md border border-divider">
        <label
          v-for="b in cleanupList"
          :key="b"
          class="flex cursor-pointer items-center gap-2 border-b border-divider-light px-2.5 py-1.5 last:border-0 hover:bg-canvas"
        >
          <Checkbox :model-value="cleanupSelected.has(b)" binary @change="toggleCleanup(b)" />
          <i class="pi pi-sitemap text-xs text-muted" />
          <span class="min-w-0 flex-1 truncate text-sm text-ink">{{ b }}</span>
          <span class="shrink-0 rounded-full bg-red-100 px-1.5 text-[10px] font-bold text-red-700">gone</span>
        </label>
      </div>
    </div>
    <template #footer>
      <Button size="small" outlined severity="secondary" @click="visible = false">Đóng</Button>
      <Button
        size="small"
        severity="danger"
        :disabled="!cleanupSelected.size"
        @click="doCleanup"
      >
        <i class="pi pi-trash mr-1.5" /> Xóa {{ cleanupSelected.size }} branch
      </Button>
    </template>
  </Dialog>
</template>
