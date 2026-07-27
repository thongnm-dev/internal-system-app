<script setup lang="ts">
import { ref } from "vue";
import Button from "primevue/button";
import Dialog from "primevue/dialog";
import InputText from "primevue/inputtext";
import type { GitApi } from "../composables/useGit";

const props = defineProps<{ git: GitApi }>();
const visible = defineModel<boolean>("visible", { default: false });

const newBranchName = ref("");

async function doCreateBranch() {
  const name = newBranchName.value.trim();
  if (!name) return;
  await props.git.createBranch(name);
  visible.value = false;
  newBranchName.value = "";
}
</script>

<template>
  <Dialog v-model:visible="visible" modal header="Tạo branch mới" :style="{ width: '420px' }">
    <div class="flex flex-col gap-3">
      <label class="text-sm font-medium text-ink">Tên branch</label>
      <InputText
        v-model="newBranchName"
        placeholder="feature/ten-branch"
        class="w-full"
        @keydown.enter="doCreateBranch"
      />
      <p class="text-xs text-muted">
        Tạo từ branch hiện tại (<strong>{{ git.info.value?.current_branch }}</strong>) và tự động chuyển sang.
      </p>
    </div>
    <template #footer>
      <Button size="small" outlined severity="secondary" @click="visible = false">Hủy</Button>
      <Button size="small" :disabled="!newBranchName.trim()" @click="doCreateBranch">
        <i class="pi pi-plus mr-1.5" /> Tạo branch
      </Button>
    </template>
  </Dialog>
</template>
