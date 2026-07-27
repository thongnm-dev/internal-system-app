<script setup lang="ts">
import { ref } from "vue";
import Button from "primevue/button";
import Dialog from "primevue/dialog";
import InputText from "primevue/inputtext";
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
      <Button size="small" outlined severity="secondary" @click="visible = false">Hủy</Button>
      <Button size="small" :loading="git.syncing.value" :disabled="!cloneUrl.trim()" @click="doClone">
        <i class="pi pi-cloud-download mr-1.5" /> Clone
      </Button>
    </template>
  </Dialog>
</template>
