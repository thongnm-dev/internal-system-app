<script setup lang="ts">
import Button from "primevue/button";
import Dialog from "primevue/dialog";
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
      <Button size="small" outlined severity="secondary" @click="visible = false">Hủy</Button>
      <Button size="small" severity="danger" :disabled="!!git.busyMessage.value" @click="doResetHard">
        <i class="pi pi-exclamation-triangle mr-1.5" /> Reset hard
      </Button>
    </template>
  </Dialog>
</template>
