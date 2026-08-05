<script setup lang="ts">
import { ref, watch } from "vue";
import Dialog from "primevue/dialog";
import InputText from "primevue/inputtext";
import DialogFooter from "@/shared/components/DialogFooter.vue";
import type { GitApi } from "../composables/useGit";
import type { GitCommit } from "@/_/types/git";

const props = defineProps<{ git: GitApi; target: GitCommit | null }>();
const visible = defineModel<boolean>("visible", { default: false });

const branchFromName = ref("");

watch(visible, (v) => {
  if (v) branchFromName.value = "";
});

async function doBranchFrom() {
  if (!props.target || !branchFromName.value.trim()) return;
  await props.git.createBranchAt(branchFromName.value, props.target.hash);
  visible.value = false;
  branchFromName.value = "";
}
</script>

<template>
  <Dialog v-model:visible="visible" modal header="Tạo branch từ commit" :style="{ width: '440px' }">
    <div class="flex flex-col gap-3">
      <div v-if="target" class="rounded-md border border-divider bg-canvas p-2.5">
        <p class="text-sm font-medium text-ink">{{ target.subject }}</p>
        <p class="mt-0.5 font-mono text-[11px] text-muted">{{ target.short_hash }}</p>
      </div>
      <div>
        <label class="mb-1 block text-xs font-bold text-muted">Tên branch</label>
        <InputText
          v-model="branchFromName"
          placeholder="feature/ten-branch"
          class="w-full"
          @keydown.enter="doBranchFrom"
        />
      </div>
    </div>
    <template #footer>
      <DialogFooter
        cancel-label="Hủy"
        confirm-label="Tạo branch"
        confirm-icon="pi pi-sitemap"
        :confirm-disabled="!branchFromName.trim()"
        @cancel="visible = false"
        @confirm="doBranchFrom"
      />
    </template>
  </Dialog>
</template>
