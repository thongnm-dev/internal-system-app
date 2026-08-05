<script setup lang="ts">
import Dialog from "primevue/dialog";
import DialogFooter from "@/shared/components/DialogFooter.vue";
import type { GitApi } from "../composables/useGit";

const props = defineProps<{
  git: GitApi;
  target: { files: string[]; label: string } | null;
}>();
const visible = defineModel<boolean>("visible", { default: false });

async function confirmDiscard() {
  if (props.target) await props.git.discardFiles(props.target.files);
  visible.value = false;
}
</script>

<template>
  <Dialog v-model:visible="visible" modal header="Xác nhận discard" :style="{ width: '420px' }">
    <p class="text-sm text-secondary">
      Bỏ thay đổi của <strong class="text-ink">{{ target?.label }}</strong>?
      Thao tác này không thể hoàn tác.
    </p>
    <template #footer>
      <DialogFooter
        cancel-label="Hủy"
        confirm-label="Discard"
        confirm-icon="pi pi-trash"
        confirm-severity="danger"
        @cancel="visible = false"
        @confirm="confirmDiscard"
      />
    </template>
  </Dialog>
</template>
