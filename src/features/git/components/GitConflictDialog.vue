<script setup lang="ts">
import { watch } from "vue";
import Dialog from "primevue/dialog";
import DialogFooter from "@/shared/components/DialogFooter.vue";
import type { GitApi } from "../composables/useGit";

const props = defineProps<{ git: GitApi }>();
const visible = defineModel<boolean>("visible", { default: false });

watch(visible, (v) => {
  if (v) props.git.loadConflicts();
});

async function doFinishConflict() {
  await props.git.finishConflict();
  if (!props.git.conflicts.value.length) visible.value = false;
}
</script>

<template>
  <Dialog v-model:visible="visible" modal header="Xử lý xung đột" :style="{ width: '640px' }">
    <div class="flex flex-col gap-2">
      <p class="text-xs text-muted">
        Chọn phía giữ lại cho từng file, hoặc tự sửa file trong editor rồi bấm "Đã xử lý". Khi hết xung đột, bấm "Hoàn tất".
      </p>
      <div v-if="!git.conflicts.value.length" class="p-5 text-center text-sm text-muted">
        <i class="pi pi-check-circle mr-1.5 text-emerald-500" /> Không còn file xung đột. Bấm "Hoàn tất" để kết thúc.
      </div>
      <div v-else class="max-h-80 overflow-y-auto rounded-md border border-divider">
        <div
          v-for="f in git.conflicts.value"
          :key="f"
          class="flex items-center gap-2 border-b border-divider-light px-2.5 py-2 last:border-0"
        >
          <i class="pi pi-exclamation-triangle shrink-0 text-xs text-red-500" />
          <span class="min-w-0 flex-1 truncate font-mono text-xs text-ink" :title="f">{{ f }}</span>
          <button
            class="shrink-0 rounded border border-divider px-2 py-0.5 text-[11px] text-secondary transition-colors hover:border-brand hover:text-brand"
            title="Giữ bản HEAD (ours)"
            @click="git.resolveConflict(f, 'ours')"
          >
            Giữ HEAD
          </button>
          <button
            class="shrink-0 rounded border border-divider px-2 py-0.5 text-[11px] text-secondary transition-colors hover:border-brand hover:text-brand"
            title="Giữ bản đến (theirs)"
            @click="git.resolveConflict(f, 'theirs')"
          >
            Giữ bản đến
          </button>
          <button
            class="shrink-0 rounded border border-divider px-2 py-0.5 text-[11px] text-secondary transition-colors hover:border-brand hover:text-brand"
            title="Đã tự sửa xong (stage file)"
            @click="git.markResolved(f)"
          >
            Đã xử lý
          </button>
        </div>
      </div>
    </div>
    <template #footer>
      <DialogFooter
        cancel-label="Đóng"
        confirm-label="Hoàn tất"
        confirm-icon="pi pi-check"
        :confirm-disabled="!!git.conflicts.value.length || !!git.busyMessage.value"
        @cancel="visible = false"
        @confirm="doFinishConflict"
      />
    </template>
  </Dialog>
</template>
