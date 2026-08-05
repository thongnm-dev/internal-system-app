<script setup lang="ts">
import { ref } from "vue";
import Dialog from "primevue/dialog";
import InputText from "primevue/inputtext";
import DialogFooter from "@/shared/components/DialogFooter.vue";
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
      <label class="text-xs font-bold text-muted">Tên branch</label>
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
      <DialogFooter
        cancel-label="Hủy"
        confirm-label="Tạo branch"
        confirm-icon="pi pi-plus"
        :confirm-disabled="!newBranchName.trim()"
        @cancel="visible = false"
        @confirm="doCreateBranch"
      />
    </template>
  </Dialog>
</template>
