<script setup lang="ts">
import { ref } from "vue";
import Dialog from "primevue/dialog";
import InputText from "primevue/inputtext";
import DialogFooter from "@/shared/components/DialogFooter.vue";
import type { GitApi } from "../composables/useGit";

const props = defineProps<{ git: GitApi }>();
const visible = defineModel<boolean>("visible", { default: false });

const cloneUrl = ref("");

async function doClone() {
  const ok = await props.git.cloneRepo(cloneUrl.value);
  if (ok) {
    visible.value = false;
    cloneUrl.value = "";
  }
}
</script>

<template>
  <Dialog v-model:visible="visible" modal header="Clone repository" :style="{ width: '460px' }">
    <div class="flex flex-col gap-3">
      <label class="text-sm font-medium text-ink">URL repository</label>
      <InputText
        v-model="cloneUrl"
        placeholder="https://github.com/user/repo.git"
        class="w-full"
        @keydown.enter="doClone"
      />
      <p class="text-xs text-muted">Sau khi nhập URL, bạn sẽ chọn thư mục để clone vào.</p>
    </div>
    <template #footer>
      <DialogFooter
        cancel-label="Hủy"
        confirm-label="Clone"
        confirm-icon="pi pi-cloud-download"
        :busy="git.syncing.value"
        :confirm-disabled="!cloneUrl.trim()"
        @cancel="visible = false"
        @confirm="doClone"
      />
    </template>
  </Dialog>
</template>
