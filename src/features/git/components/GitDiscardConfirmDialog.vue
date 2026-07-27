<script setup lang="ts">
import Button from "primevue/button";
import Dialog from "primevue/dialog";
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
      <Button size="small" outlined severity="secondary" @click="visible = false">Hủy</Button>
      <Button size="small" severity="danger" @click="confirmDiscard">
        <i class="pi pi-trash mr-1.5" /> Discard
      </Button>
    </template>
  </Dialog>
</template>
