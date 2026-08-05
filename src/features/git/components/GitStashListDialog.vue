<script setup lang="ts">
import { watch } from "vue";
import Button from "primevue/button";
import Dialog from "primevue/dialog";
import DialogFooter from "@/shared/components/DialogFooter.vue";
import type { GitApi } from "../composables/useGit";

const props = defineProps<{ git: GitApi }>();
const visible = defineModel<boolean>("visible", { default: false });

watch(visible, (v) => {
  if (v) props.git.refreshStashes();
});
</script>

<template>
  <Dialog v-model:visible="visible" modal header="Stash" :style="{ width: '620px' }">
    <div class="flex flex-col gap-2">
      <div
        v-for="s in git.stashes.value"
        :key="s.reference"
        class="flex items-center gap-2 rounded-md border border-divider px-3 py-2"
      >
        <i class="pi pi-inbox shrink-0 text-sm text-muted" />
        <div class="min-w-0 flex-1">
          <p class="truncate text-xs text-ink">{{ s.message || s.reference }}</p>
          <p class="font-mono text-[11px] text-muted">{{ s.reference }}</p>
        </div>
        <Button
          size="small"
          outlined
          severity="secondary"
          class="shrink-0"
          title="Áp dụng (giữ lại stash)"
          @click="git.stashApply(s.reference, false)"
        >
          <i class="pi pi-replay" />
        </Button>
        <Button
          size="small"
          outlined
          class="shrink-0"
          title="Áp dụng và xóa stash (pop)"
          @click="git.stashApply(s.reference, true)"
        >
          <i class="pi pi-upload" />
        </Button>
        <Button
          size="small"
          outlined
          severity="danger"
          class="shrink-0"
          title="Xóa stash"
          @click="git.stashDrop(s.reference)"
        >
          <i class="pi pi-trash" />
        </Button>
      </div>
      <div v-if="!git.stashes.value.length" class="p-4 text-center text-sm text-muted">
        Chưa có stash nào.
      </div>
    </div>
    <template #footer>
      <DialogFooter cancel-label="Đóng" hide-confirm @cancel="visible = false" />
    </template>
  </Dialog>
</template>
