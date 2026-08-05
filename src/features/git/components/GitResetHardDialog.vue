<script setup lang="ts">
import Dialog from "primevue/dialog";
import DialogFooter from "@/shared/components/DialogFooter.vue";
import type { GitApi } from "../composables/useGit";
import type { GitCommit } from "@/_/types/git";

const props = defineProps<{ git: GitApi; target: GitCommit | null }>();
const visible = defineModel<boolean>("visible", { default: false });

async function doResetHard() {
  if (props.target) await props.git.resetTo(props.target.hash, "hard");
  visible.value = false;
}
</script>

<template>
  <Dialog v-model:visible="visible" modal header="Reset (hard)" :style="{ width: '470px' }">
    <p class="text-sm text-secondary">
      Reset branch về <strong class="text-ink">{{ target?.short_hash }}</strong> và
      <strong class="text-red-600">xóa toàn bộ thay đổi</strong> sau commit này (kể cả file đang sửa).
      Thao tác này không thể hoàn tác.
    </p>
    <template #footer>
      <DialogFooter
        cancel-label="Hủy"
        confirm-label="Reset hard"
        confirm-icon="pi pi-exclamation-triangle"
        confirm-severity="danger"
        :confirm-disabled="!!git.busyMessage.value"
        @cancel="visible = false"
        @confirm="doResetHard"
      />
    </template>
  </Dialog>
</template>
