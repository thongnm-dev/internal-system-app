<script setup lang="ts">
import Dialog from "primevue/dialog";
import DialogFooter from "@/shared/components/DialogFooter.vue";
import type { GitApi } from "../composables/useGit";
import type { GitCommit } from "@/_/types/git";

const props = defineProps<{ git: GitApi; target: GitCommit | null }>();
const visible = defineModel<boolean>("visible", { default: false });

async function doRevert() {
  if (!props.target) return;
  await props.git.revert(props.target.hash);
  visible.value = false;
}
</script>

<template>
  <Dialog v-model:visible="visible" modal header="Revert commit" :style="{ width: '460px' }">
    <p class="text-sm text-secondary">
      Tạo một commit mới đảo ngược thay đổi của:
    </p>
    <div v-if="target" class="mt-2 rounded-md border border-divider bg-canvas p-2.5">
      <p class="text-sm font-medium text-ink">{{ target.subject }}</p>
      <p class="mt-0.5 font-mono text-[11px] text-muted">{{ target.short_hash }} · {{ target.author_name }}</p>
    </div>
    <template #footer>
      <DialogFooter
        cancel-label="Hủy"
        confirm-label="Revert"
        confirm-icon="pi pi-undo"
        :confirm-disabled="!!git.busyMessage.value"
        @cancel="visible = false"
        @confirm="doRevert"
      />
    </template>
  </Dialog>
</template>
